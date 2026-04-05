import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';
import 'package:path/path.dart' as p;

class DemoEnginePlatform extends MsViewerPlatform {
  DemoEnginePlatform({DemoEngineRunner? runner})
    : _runner = runner ?? CargoViewerCliRunner();

  final DemoEngineRunner _runner;

  @override
  Future<OpenDocumentResult> openDocument(OpenDocumentRequest request) async {
    try {
      final payload = await _runner.run(
        'open-document',
        jsonEncode(request.toJson()),
      );
      return OpenDocumentResult.fromJson(_decodeMap(payload));
    } catch (error) {
      return OpenDocumentFailure(_platformError(error));
    }
  }

  @override
  Future<GetPageRenderModelResult> getPageRenderModel(
    GetPageRenderModelRequest request,
  ) async {
    try {
      final payload = await _runner.run(
        'get-page-render-model',
        jsonEncode(request.toJson()),
      );
      return GetPageRenderModelResult.fromJson(_decodeMap(payload));
    } catch (error) {
      return GetPageRenderModelFailure(_platformError(error));
    }
  }

  @override
  Future<SearchDocumentResult> searchDocument(
    SearchDocumentRequest request,
  ) async {
    try {
      final payload = await _runner.run(
        'search-document',
        jsonEncode(request.toJson()),
      );
      return SearchDocumentResult.fromJson(jsonDecode(payload));
    } catch (error) {
      return SearchDocumentFailure(_platformError(error));
    }
  }

  @override
  Future<GetSelectionPageResult> getSelectionPage(
    GetSelectionPageRequest request,
  ) async {
    try {
      final payload = await _runner.run(
        'get-selection-page',
        jsonEncode(request.toJson()),
      );
      return GetSelectionPageResult.fromJson(_decodeMap(payload));
    } catch (error) {
      return GetSelectionPageFailure(_platformError(error));
    }
  }

  Map<String, Object?> _decodeMap(String payload) {
    return (jsonDecode(payload) as Map).cast<String, Object?>();
  }
}

abstract class DemoEngineRunner {
  Future<String> run(String command, String requestJson);
}

class CargoViewerCliRunner extends DemoEngineRunner {
  CargoViewerCliRunner();

  Future<void> _queue = Future<void>.value();
  Process? _serverProcess;
  StreamIterator<String>? _stdoutLines;
  final StringBuffer _stderrBuffer = StringBuffer();

  @override
  Future<String> run(String command, String requestJson) {
    final completer = Completer<String>();
    _queue = _queue.then((_) async {
      try {
        final process = await _ensureServerProcess();
        final stdoutLines = _stdoutLines;
        if (stdoutLines == null) {
          throw StateError('viewer_cli stdout stream is not initialized.');
        }

        process.stdin.writeln(
          jsonEncode({
            'command': command,
            'requestJson': requestJson,
          }),
        );
        await process.stdin.flush();

        final hasNext = await stdoutLines.moveNext();
        if (!hasNext) {
          throw ProcessException(
            _resolvedBinaryPath(await _workspaceRoot()),
            ['serve'],
            _stderrMessage('viewer_cli server exited unexpectedly.'),
          );
        }

        completer.complete(stdoutLines.current.trim());
      } catch (error, stackTrace) {
        completer.completeError(error, stackTrace);
      }
    });
    return completer.future;
  }

  Future<Process> _ensureServerProcess() async {
    final existing = _serverProcess;
    if (existing != null) {
      return existing;
    }

    if (!_supportsCliBridge) {
      throw ProcessException(
        'viewer_cli',
        const [],
        'Rust CLI bridge is only available on desktop platforms.',
      );
    }

    final workspaceRoot = await _workspaceRoot();
    final manifestPath = p.join(workspaceRoot.path, 'rust', 'Cargo.toml');
    final buildResult = await Process.run('cargo', [
      'build',
      '--quiet',
      '--manifest-path',
      manifestPath,
      '-p',
      'viewer_cli',
    ], workingDirectory: workspaceRoot.path);

    if (buildResult.exitCode != 0) {
      final error = (buildResult.stderr as String).trim();
      throw ProcessException(
        'cargo',
        ['build', '-p', 'viewer_cli'],
        error.isEmpty ? 'viewer_cli build failed' : error,
        buildResult.exitCode,
      );
    }

    final binaryPath = _resolvedBinaryPath(workspaceRoot);
    final process = await Process.start(binaryPath, ['serve'], workingDirectory: workspaceRoot.path);
    _stderrBuffer.clear();
    process.stderr
        .transform(utf8.decoder)
        .listen((chunk) => _stderrBuffer.write(chunk));
    _stdoutLines = StreamIterator(
      process.stdout.transform(utf8.decoder).transform(const LineSplitter()),
    );
    unawaited(
      process.exitCode.then((_) {
        _serverProcess = null;
        _stdoutLines = null;
      }),
    );
    _serverProcess = process;
    return process;
  }

  Future<Directory> _workspaceRoot() async {
    return _findWorkspaceRoot();
  }

  String _resolvedBinaryPath(Directory workspaceRoot) {
    return p.join(
      workspaceRoot.path,
      'rust',
      'target',
      'debug',
      Platform.isWindows ? 'viewer_cli.exe' : 'viewer_cli',
    );
  }

  String _stderrMessage(String fallback) {
    final message = _stderrBuffer.toString().trim();
    return message.isEmpty ? fallback : message;
  }
}

bool get _supportsCliBridge =>
    !kIsWeb &&
    (Platform.isMacOS || Platform.isWindows || Platform.isLinux);

Directory _findWorkspaceRoot() {
  final override = const String.fromEnvironment('MS_VIEWER_WORKSPACE_ROOT');
  if (override.isNotEmpty) {
    final directory = Directory(override);
    final candidate = File(p.join(directory.path, 'rust', 'Cargo.toml'));
    if (candidate.existsSync()) {
      return directory.absolute;
    }
  }

  final startDirectories = <Directory>[
    Directory.current.absolute,
    File(Platform.resolvedExecutable).absolute.parent,
    File(Platform.executable).absolute.parent,
  ];

  for (final start in startDirectories) {
    final resolved = _searchWorkspaceRootFrom(start);
    if (resolved != null) {
      return resolved;
    }
  }

  throw StateError(
    'Could not find workspace root containing rust/Cargo.toml. '
    'Tried from: ${startDirectories.map((dir) => dir.path).join(', ')}',
  );
}

Directory? _searchWorkspaceRootFrom(Directory start) {
  var current = start.absolute;
  while (true) {
    final candidate = File(p.join(current.path, 'rust', 'Cargo.toml'));
    if (candidate.existsSync()) {
      return current;
    }

    final parent = current.parent;
    if (parent.path == current.path) {
      return null;
    }
    current = parent;
  }
}

OpenDocumentError _platformError(Object error) {
  if (error is ProcessException && error.message.contains('desktop platforms')) {
    return const OpenDocumentError(
      code: ViewerErrorCode.notImplemented,
      message: 'Rust CLI bridge is only available on desktop platforms.',
    );
  }

  if (error is ProcessException && error.errorCode == 2) {
    return const OpenDocumentError(
      code: ViewerErrorCode.notImplemented,
      message: 'cargo was not found. Install Rust and ensure cargo is on PATH.',
    );
  }

  return OpenDocumentError(
    code: ViewerErrorCode.ioError,
    message: error.toString(),
  );
}
