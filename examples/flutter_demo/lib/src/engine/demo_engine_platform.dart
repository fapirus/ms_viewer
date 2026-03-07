import 'dart:convert';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';
import 'package:path/path.dart' as p;

class DemoEnginePlatform extends MsViewerPlatform {
  DemoEnginePlatform({DemoEngineRunner? runner})
    : _runner = runner ?? const CargoViewerCliRunner();

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
  const DemoEngineRunner();

  Future<String> run(String command, String requestJson);
}

class CargoViewerCliRunner extends DemoEngineRunner {
  const CargoViewerCliRunner();

  @override
  Future<String> run(String command, String requestJson) async {
    if (!_supportsCliBridge) {
      throw ProcessException(
        'cargo',
        [],
        'Rust CLI bridge is only available on desktop platforms.',
      );
    }

    final workspaceRoot = _findWorkspaceRoot();
    final manifestPath = p.join(workspaceRoot.path, 'rust', 'Cargo.toml');
    final result = await Process.run('cargo', [
      'run',
      '--quiet',
      '--manifest-path',
      manifestPath,
      '-p',
      'viewer_cli',
      '--',
      command,
      requestJson,
    ], workingDirectory: workspaceRoot.path);

    if (result.exitCode != 0) {
      final error = (result.stderr as String).trim();
      throw ProcessException(
        'cargo',
        ['run', '-p', 'viewer_cli', '--', command],
        error.isEmpty ? 'viewer_cli failed' : error,
        result.exitCode,
      );
    }

    return (result.stdout as String).trim();
  }
}

bool get _supportsCliBridge =>
    !kIsWeb &&
    (Platform.isMacOS || Platform.isWindows || Platform.isLinux);

Directory _findWorkspaceRoot() {
  var current = Directory.current.absolute;
  while (true) {
    final candidate = File(p.join(current.path, 'rust', 'Cargo.toml'));
    if (candidate.existsSync()) {
      return current;
    }

    final parent = current.parent;
    if (parent.path == current.path) {
      throw StateError('Could not find workspace root containing rust/Cargo.toml.');
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

  return OpenDocumentError(
    code: ViewerErrorCode.ioError,
    message: error.toString(),
  );
}
