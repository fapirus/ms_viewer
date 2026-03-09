import 'dart:async';

import 'package:cross_file/cross_file.dart';
import 'package:desktop_drop/desktop_drop.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:ms_viewer/ms_viewer.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart'
    as platform;

import 'demo_document.dart';
import 'demo_file_access.dart';
import 'demo_fixture_catalog.dart';
import 'demo_viewer_page.dart';

class DemoHomePage extends StatefulWidget {
  const DemoHomePage({
    super.key,
    required this.viewerPlatform,
    required this.assetBundle,
    this.pickFiles = pickDemoFiles,
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
  late final List<DemoDocumentEntry> _fixtures;
  final List<DemoDocumentEntry> _imports = [];
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
    _fixtures = buildFixtureEntries();
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
    unawaited(_openViewer(added.first));
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
    unawaited(_openViewer(added.first));
  }

  Future<void> _openViewer(DemoDocumentEntry entry) async {
    if (!mounted) {
      return;
    }

    await Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (context) => DemoViewerPage(
          entry: entry,
          viewerPlatform: widget.viewerPlatform,
          assetBundle: widget.assetBundle,
        ),
      ),
    );
  }

  DemoDocumentEntry _importedEntry(String path, DemoDocumentOrigin origin) {
    return buildImportedEntry(path, origin);
  }

  @override
  Widget build(BuildContext context) {
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
      body: DecoratedBox(
        decoration: const BoxDecoration(
          gradient: LinearGradient(
            colors: [Color(0xFFF8FBFF), Color(0xFFF3F7FC)],
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
          ),
        ),
        child: ListView(
          padding: const EdgeInsets.all(24),
          children: [
            _buildLibraryIntro(context),
            const SizedBox(height: 20),
            _buildSectionCard(
              context: context,
              title: 'Fixtures',
              description: 'DOCX, PPTX, XLSX 기준 문서를 바로 열어 엔진 상태를 점검합니다.',
              children: _fixtures.map(_documentTile).toList(growable: false),
            ),
            if (_imports.isNotEmpty) ...[
              const SizedBox(height: 16),
              _buildSectionCard(
                context: context,
                title: 'Imported Files',
                description: 'Open File 또는 drag & drop으로 추가한 문서입니다.',
                children: _imports.map(_documentTile).toList(growable: false),
              ),
            ],
            if (_supportsDesktopDrop) ...[
              const SizedBox(height: 16),
              _buildSectionCard(
                context: context,
                title: 'Desktop Drop',
                description: '데스크톱에서는 파일을 끌어다 놓아 바로 열 수 있습니다.',
                children: [_buildDropZone()],
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildLibraryIntro(BuildContext context) {
    return Card(
      elevation: 0,
      color: Colors.white,
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Container(
              width: 52,
              height: 52,
              decoration: BoxDecoration(
                color: const Color(0xFFDCEBFF),
                borderRadius: BorderRadius.circular(18),
              ),
              alignment: Alignment.center,
              child: const Icon(
                Icons.folder_copy_outlined,
                color: Color(0xFF1D4ED8),
              ),
            ),
            const SizedBox(width: 16),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'Document Library',
                    style: Theme.of(context).textTheme.headlineSmall,
                  ),
                  const SizedBox(height: 8),
                  Text(
                    '문서를 선택하면 별도 뷰어 화면에서 열립니다. 이 화면은 fixture, picked, dropped 문서의 진입점만 담당합니다.',
                    style: Theme.of(context).textTheme.bodyMedium,
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSectionCard({
    required BuildContext context,
    required String title,
    required String description,
    required List<Widget> children,
  }) {
    return Card(
      elevation: 0,
      color: Colors.white,
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: Theme.of(context).textTheme.titleLarge),
            const SizedBox(height: 6),
            Text(description, style: Theme.of(context).textTheme.bodySmall),
            const SizedBox(height: 16),
            ...children,
          ],
        ),
      ),
    );
  }

  Widget _documentTile(DemoDocumentEntry entry) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Material(
        color: const Color(0xFFF8FBFF),
        borderRadius: BorderRadius.circular(20),
        child: InkWell(
          borderRadius: BorderRadius.circular(20),
          onTap: () => unawaited(_openViewer(entry)),
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Row(
              children: [
                Container(
                  width: 44,
                  height: 44,
                  decoration: BoxDecoration(
                    color: Colors.white,
                    borderRadius: BorderRadius.circular(14),
                  ),
                  alignment: Alignment.center,
                  child: Icon(
                    _iconForKind(entry.kind),
                    color: const Color(0xFF1D4ED8),
                  ),
                ),
                const SizedBox(width: 14),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        entry.title,
                        style: const TextStyle(fontWeight: FontWeight.w600),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        entry.statusLabel,
                        style: Theme.of(context).textTheme.bodySmall,
                      ),
                    ],
                  ),
                ),
                if (entry.requiresPassword)
                  const Padding(
                    padding: EdgeInsets.only(right: 8),
                    child: Icon(Icons.lock_outline, size: 18),
                  ),
                const Icon(Icons.chevron_right),
              ],
            ),
          ),
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
          color: _dragging ? const Color(0xFFDCEBFF) : const Color(0xFFF8FBFF),
          borderRadius: BorderRadius.circular(20),
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
            Text(
              'Drop docx, pptx, or xlsx files here to open them in the viewer.',
            ),
          ],
        ),
      ),
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
