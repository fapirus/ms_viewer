import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:ms_viewer/ms_viewer.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart'
    as platform;

import 'demo_document.dart';
import 'demo_file_access.dart';

class DemoViewerPage extends StatefulWidget {
  const DemoViewerPage({
    super.key,
    required this.entry,
    required this.viewerPlatform,
    required this.assetBundle,
    this.pickFiles = pickDemoFiles,
  });

  final DemoDocumentEntry entry;
  final platform.MsViewerPlatform viewerPlatform;
  final AssetBundle assetBundle;
  final DemoFilePicker pickFiles;

  @override
  State<DemoViewerPage> createState() => _DemoViewerPageState();
}

class _DemoViewerPageState extends State<DemoViewerPage> {
  late final MsViewerController _controller;

  @override
  void initState() {
    super.initState();
    _controller = MsViewerController(platform: widget.viewerPlatform);
    unawaited(_openEntry());
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _openEntry() async {
    final entry = widget.entry;
    if (entry.origin == DemoDocumentOrigin.fixture &&
        _usesEngineForFixture(entry.kind) &&
        entry.location != null) {
      final bytes = await widget.assetBundle.load(entry.location!);
      final encoded = base64Encode(bytes.buffer.asUint8List());
      await _controller.openDocument(
        platform.OpenDocumentRequest(
          source: platform.OpenDocumentSource.bytesBase64(encoded),
        ),
      );
      return;
    }

    if (_usesEngineForImportedPath(entry) && entry.location != null) {
      await _controller.openDocument(
        platform.OpenDocumentRequest(
          source: platform.OpenDocumentSource.path(entry.location!),
        ),
      );
      return;
    }

    _controller.attachDocument(entry.descriptor);
  }

  bool _usesEngineForFixture(DocumentKind kind) {
    return kind == DocumentKind.docx ||
        kind == DocumentKind.pptx ||
        kind == DocumentKind.xlsx;
  }

  bool _usesEngineForPickedImport(DocumentKind kind) {
    return kind == DocumentKind.docx ||
        kind == DocumentKind.pptx ||
        kind == DocumentKind.xlsx;
  }

  bool _usesEngineForImportedPath(DemoDocumentEntry entry) {
    if (entry.origin == DemoDocumentOrigin.picked) {
      return _usesEngineForPickedImport(entry.kind);
    }

    if (entry.origin == DemoDocumentOrigin.dropped) {
      return entry.kind == DocumentKind.docx ||
          entry.kind == DocumentKind.pptx ||
          entry.kind == DocumentKind.xlsx;
    }

    return false;
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.entry.title),
        actions: [
          FilledButton.icon(
            onPressed: _openReplacementFile,
            icon: const Icon(Icons.upload_file),
            label: const Text('Open File'),
          ),
          const SizedBox(width: 16),
        ],
      ),
      body: DecoratedBox(
        decoration: const BoxDecoration(
          gradient: LinearGradient(
            colors: [Color(0xFFF8FBFF), Color(0xFFF3F7FC)],
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
          ),
        ),
        child: Column(
          children: [
            if (widget.entry.kind != DocumentKind.xlsx)
              _buildViewerChrome(context),
            Expanded(
              child: widget.entry.requiresPassword
                  ? _buildLockedPreview(context)
                  : MsDocumentView(
                      controller: _controller,
                      previewPages: widget.entry.previewPages,
                    ),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _openReplacementFile() async {
    final paths = await widget.pickFiles();
    if (paths.isEmpty || !mounted) {
      return;
    }

    final replacement = buildImportedEntry(
      paths.first,
      DemoDocumentOrigin.picked,
    );

    await Navigator.of(context).pushReplacement(
      MaterialPageRoute<void>(
        builder: (context) => DemoViewerPage(
          entry: replacement,
          viewerPlatform: widget.viewerPlatform,
          assetBundle: widget.assetBundle,
          pickFiles: widget.pickFiles,
        ),
      ),
    );
  }

  Widget _buildViewerChrome(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(24, 24, 24, 0),
      child: Card(
        elevation: 0,
        color: Colors.white,
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  _metaChip(widget.entry.sourceLabel),
                  _metaChip(widget.entry.statusLabel),
                  ...widget.entry.tags.map(_metaChip),
                ],
              ),
              if (widget.entry.location != null) ...[
                const SizedBox(height: 10),
                SelectableText(
                  widget.entry.location!,
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
              if (widget.entry.note != null) ...[
                const SizedBox(height: 10),
                Text(
                  widget.entry.note!,
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildLockedPreview(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.lock_outline, size: 48, color: Color(0xFF1D4ED8)),
            const SizedBox(height: 16),
            Text(
              'Password required fixture',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            const Text(
              'This bundled file represents the encrypted DOCX flow. The real password UI is already wired in the viewer shell, and engine-side decryption remains a follow-up task.',
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }

  Widget _metaChip(String label) {
    return Chip(
      label: Text(label),
      visualDensity: VisualDensity.compact,
      side: const BorderSide(color: Color(0xFFBFDBFE)),
      backgroundColor: Colors.white,
    );
  }
}
