import 'dart:async';
import 'dart:convert';

import 'package:cross_file/cross_file.dart';
import 'package:desktop_drop/desktop_drop.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter/material.dart';
import 'package:ms_viewer/ms_viewer.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart'
    as platform;
import 'package:path/path.dart' as p;

import 'demo_document.dart';
import 'demo_fixture_catalog.dart';
import 'demo_page_models.dart';

typedef DemoFilePicker = Future<List<String>> Function();

class DemoHomePage extends StatefulWidget {
  const DemoHomePage({
    super.key,
    required this.viewerPlatform,
    required this.assetBundle,
    this.pickFiles = _defaultPickFiles,
    this.supportsDesktopDropOverride,
  });

  final platform.MsViewerPlatform viewerPlatform;
  final AssetBundle assetBundle;
  final DemoFilePicker pickFiles;
  final bool? supportsDesktopDropOverride;

  @override
  State<DemoHomePage> createState() => _DemoHomePageState();
}

class _DemoHomePageState extends State<DemoHomePage> {
  late final MsViewerController _controller;
  late final List<DemoDocumentEntry> _fixtures;
  final List<DemoDocumentEntry> _imports = [];
  DemoDocumentEntry? _selected;
  bool _dragging = false;

  bool get _supportsDesktopDrop =>
      widget.supportsDesktopDropOverride ??
      (!kIsWeb &&
          (defaultTargetPlatform == TargetPlatform.macOS ||
              defaultTargetPlatform == TargetPlatform.windows ||
              defaultTargetPlatform == TargetPlatform.linux));

  @override
  void initState() {
    super.initState();
    _controller = MsViewerController(platform: widget.viewerPlatform);
    _fixtures = buildFixtureEntries();
    unawaited(_selectEntry(_fixtures.first));
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _pickFiles() async {
    final paths = await widget.pickFiles();
    final added = paths
        .map((path) => _importedEntry(path, DemoDocumentOrigin.picked))
        .toList(growable: false);
    if (added.isEmpty) {
      return;
    }

    setState(() {
      _imports.insertAll(0, added.reversed);
    });
    unawaited(_selectEntry(added.first));
  }

  Future<void> _openDroppedFiles(List<XFile> files) async {
    await handleDroppedPaths(
      files.map((file) => file.path).where((path) => path.isNotEmpty).toList(),
    );
  }

  Future<void> handleDroppedPaths(List<String> paths) async {
    final added = paths
        .map((path) => _importedEntry(path, DemoDocumentOrigin.dropped))
        .toList(growable: false);
    if (added.isEmpty) {
      return;
    }

    setState(() {
      _imports.insertAll(0, added.reversed);
      _dragging = false;
    });
    await _selectEntry(added.first);
  }

  DemoDocumentEntry _importedEntry(String path, DemoDocumentOrigin origin) {
    final extension = p.extension(path).replaceFirst('.', '').toLowerCase();
    final title = p.basename(path);
    final kind = switch (extension) {
      'pptx' => DocumentKind.pptx,
      'xlsx' => DocumentKind.xlsx,
      _ => DocumentKind.docx,
    };
    final note = switch (extension) {
      'docx' =>
        'External DOCX file. Real engine open is wired in the desktop demo.',
      'pptx' => 'PPTX parser and renderer are not implemented yet.',
      'xlsx' => 'XLSX parser and renderer are not implemented yet.',
      _ => 'Unknown document type.',
    };

    return DemoDocumentEntry(
      id: '${origin.name}:$path',
      title: title,
      kind: kind,
      origin: origin,
      previewPages: [
        buildImportedPreviewPage(title: title, extension: extension),
      ],
      tags: [extension.isEmpty ? 'file' : extension, origin.name],
      sourceLabel: origin == DemoDocumentOrigin.picked
          ? 'Picked file'
          : 'Dropped file',
      statusLabel: 'Ready',
      location: path,
      note: note,
    );
  }

  Future<void> _selectEntry(DemoDocumentEntry entry) async {
    setState(() {
      _selected = entry;
    });

    if (entry.origin == DemoDocumentOrigin.fixture &&
        entry.kind == DocumentKind.docx &&
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

    if ((entry.origin == DemoDocumentOrigin.picked ||
            entry.origin == DemoDocumentOrigin.dropped) &&
        entry.kind == DocumentKind.docx &&
        entry.location != null) {
      await _controller.openDocument(
        platform.OpenDocumentRequest(
          source: platform.OpenDocumentSource.path(entry.location!),
        ),
      );
      return;
    }

    _controller.attachDocument(entry.descriptor);
  }

  @override
  Widget build(BuildContext context) {
    final selected = _selected;
    if (selected == null) {
      return const SizedBox.shrink();
    }

    return Scaffold(
      appBar: AppBar(
        title: const Text('MS Viewer Demo'),
        actions: [
          FilledButton.icon(
            onPressed: _pickFiles,
            icon: const Icon(Icons.upload_file),
            label: const Text('Open File'),
          ),
          const SizedBox(width: 16),
        ],
      ),
      body: LayoutBuilder(
        builder: (context, constraints) {
          final wide = constraints.maxWidth >= 960;
          final rail = _buildSourceRail(context, selected);
          final preview = _buildPreviewPane(
            context,
            selected,
            scrollable: !wide,
          );

          if (wide) {
            return Row(
              children: [
                SizedBox(width: 320, child: rail),
                const VerticalDivider(width: 1),
                Expanded(child: preview),
              ],
            );
          }

          return Column(
            children: [
              SizedBox(height: 320, child: rail),
              const Divider(height: 1),
              Expanded(child: preview),
            ],
          );
        },
      ),
    );
  }

  Widget _buildSourceRail(BuildContext context, DemoDocumentEntry selected) {
    final children = <Widget>[
      Padding(
        padding: const EdgeInsets.fromLTRB(20, 20, 20, 12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Fixtures', style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 6),
            Text(
              'DOCX review fixtures are bundled here for mid-project checks.',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
      ),
      ..._fixtures.map((entry) => _documentTile(entry, selected)),
      if (_imports.isNotEmpty) ...[
        const Divider(height: 24),
        Padding(
          padding: const EdgeInsets.fromLTRB(20, 0, 20, 12),
          child: Text(
            'Imported Files',
            style: Theme.of(context).textTheme.titleMedium,
          ),
        ),
        ..._imports.map((entry) => _documentTile(entry, selected)),
      ],
      if (_supportsDesktopDrop) const Divider(height: 24),
      if (_supportsDesktopDrop)
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
          child: _buildDropZone(),
        ),
      const SizedBox(height: 16),
    ];

    return ColoredBox(
      color: const Color(0xFFF8FBFF),
      child: ListView(children: children),
    );
  }

  Widget _documentTile(DemoDocumentEntry entry, DemoDocumentEntry selected) {
    final active = entry.id == selected.id;
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      child: Card(
        elevation: active ? 0 : 0,
        color: active ? const Color(0xFFE0ECFF) : Colors.white,
        child: ListTile(
          leading: Icon(
            _iconForKind(entry.kind),
            color: const Color(0xFF1D4ED8),
          ),
          title: Text(entry.title),
          subtitle: Text(entry.statusLabel),
          trailing: entry.requiresPassword
              ? const Icon(Icons.lock_outline)
              : null,
          onTap: () => unawaited(_selectEntry(entry)),
        ),
      ),
    );
  }

  Widget _buildDropZone() {
    return DropTarget(
      onDragEntered: (_) => setState(() => _dragging = true),
      onDragExited: (_) => setState(() => _dragging = false),
      onDragDone: (details) => _openDroppedFiles(details.files),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 180),
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: _dragging ? const Color(0xFFDCEBFF) : Colors.white,
          borderRadius: BorderRadius.circular(16),
          border: Border.all(
            color: _dragging
                ? const Color(0xFF2563EB)
                : const Color(0xFFBFDBFE),
            width: 1.5,
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: const [
            Icon(Icons.file_download_outlined, color: Color(0xFF2563EB)),
            SizedBox(height: 8),
            Text('Desktop Drop'),
            SizedBox(height: 4),
            Text('Drop docx, pptx, or xlsx files here to preview them.'),
          ],
        ),
      ),
    );
  }

  Widget _buildPreviewPane(
    BuildContext context,
    DemoDocumentEntry entry, {
    required bool scrollable,
  }) {
    final previewBody = entry.requiresPassword
        ? _buildLockedPreview(context)
        : MsDocumentView(
            controller: _controller,
            previewPages: entry.previewPages,
          );

    final header = Card(
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(entry.title, style: Theme.of(context).textTheme.headlineSmall),
            const SizedBox(height: 8),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                _metaChip(entry.sourceLabel),
                _metaChip(entry.statusLabel),
                ...entry.tags.map(_metaChip),
              ],
            ),
            if (entry.location != null) ...[
              const SizedBox(height: 12),
              SelectableText(
                entry.location!,
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ],
            if (entry.note != null) ...[
              const SizedBox(height: 12),
              Text(entry.note!),
            ],
          ],
        ),
      ),
    );

    if (scrollable) {
      return ListView(
        padding: const EdgeInsets.all(24),
        children: [
          header,
          const SizedBox(height: 16),
          SizedBox(height: 640, child: previewBody),
        ],
      );
    }

    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          header,
          const SizedBox(height: 16),
          Expanded(child: previewBody),
        ],
      ),
    );
  }

  Widget _buildLockedPreview(BuildContext context) {
    return Card(
      color: const Color(0xFFF8FAFC),
      child: Center(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(
                Icons.lock_outline,
                size: 48,
                color: Color(0xFF1D4ED8),
              ),
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

  IconData _iconForKind(DocumentKind kind) {
    return switch (kind) {
      DocumentKind.docx => Icons.description_outlined,
      DocumentKind.pptx => Icons.slideshow_outlined,
      DocumentKind.xlsx => Icons.table_chart_outlined,
    };
  }
}

Future<List<String>> _defaultPickFiles() async {
  final result = await FilePicker.platform.pickFiles(
    allowMultiple: true,
    type: FileType.custom,
    allowedExtensions: const ['docx', 'pptx', 'xlsx'],
  );
  if (result == null) {
    return const [];
  }

  return result.files
      .map((file) => file.path)
      .whereType<String>()
      .toList(growable: false);
}
