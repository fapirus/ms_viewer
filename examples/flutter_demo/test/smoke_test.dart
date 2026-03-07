import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_demo/main.dart';
import 'package:flutter_demo/src/demo_home_page.dart';
import 'package:flutter/services.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart'
    as platform;

void main() {
  testWidgets('demo app renders fixture browser shell', (tester) async {
    final fakePlatform = _FakeDemoPlatform();
    await tester.pumpWidget(
      DemoApp(
        platform: fakePlatform,
        assetBundle: _FakeAssetBundle(),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('MS Viewer Demo'), findsOneWidget);
    expect(find.text('Fixtures'), findsOneWidget);
    expect(find.text('Open File'), findsOneWidget);
    expect(find.text('docx_plain_text.docx'), findsWidgets);
    expect(find.text('Bundled fixture'), findsWidgets);
    expect(fakePlatform.openCallCount, 1);
    expect(fakePlatform.pageCallCount, 1);
    expect(
      fakePlatform.openSourceKinds.single,
      platform.DocumentSourceKind.bytesBase64,
    );
  });

  testWidgets('demo app opens picked docx through path source', (tester) async {
    final fakePlatform = _FakeDemoPlatform();
    await tester.pumpWidget(
      MaterialApp(
        home: DemoHomePage(
          viewerPlatform: fakePlatform,
          assetBundle: _FakeAssetBundle(),
          pickFiles: () async => const ['/tmp/picked.docx'],
          supportsDesktopDropOverride: true,
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Open File'));
    await tester.pumpAndSettle();

    expect(fakePlatform.openCallCount, 2);
    expect(fakePlatform.openSourceKinds.last, platform.DocumentSourceKind.path);
    expect(find.text('picked.docx'), findsWidgets);
  });

  testWidgets('demo app opens dropped docx through path source', (tester) async {
    final fakePlatform = _FakeDemoPlatform();
    await tester.pumpWidget(
      MaterialApp(
        home: DemoHomePage(
          viewerPlatform: fakePlatform,
          assetBundle: _FakeAssetBundle(),
          pickFiles: () async => const [],
        ),
      ),
    );
    await tester.pumpAndSettle();

    final state = tester.state(find.byType(DemoHomePage)) as dynamic;
    await state.handleDroppedPaths(const ['/tmp/dropped.docx']);
    await tester.pumpAndSettle();

    expect(fakePlatform.openCallCount, 2);
    expect(fakePlatform.openSourceKinds.last, platform.DocumentSourceKind.path);
    expect(find.text('dropped.docx'), findsWidgets);
  });

  testWidgets('demo app opens fixture pptx through bytes source', (tester) async {
    final fakePlatform = _FakeDemoPlatform();
    await tester.pumpWidget(
      DemoApp(
        platform: fakePlatform,
        assetBundle: _FakeAssetBundle(),
      ),
    );
    await tester.pumpAndSettle();

    await tester.scrollUntilVisible(
      find.text('pptx_text_shapes.pptx'),
      200,
      scrollable: find.byType(Scrollable).first,
    );
    await tester.tap(find.text('pptx_text_shapes.pptx'));
    await tester.pumpAndSettle();

    expect(fakePlatform.openCallCount, 2);
    expect(fakePlatform.openSourceKinds.last, platform.DocumentSourceKind.bytesBase64);
    expect(find.text('pptx_text_shapes.pptx'), findsWidgets);
  });
}

class _FakeDemoPlatform extends platform.MsViewerPlatform {
  int openCallCount = 0;
  int pageCallCount = 0;
  final List<platform.DocumentSourceKind> openSourceKinds = [];

  @override
  Future<platform.OpenDocumentResult> openDocument(
    platform.OpenDocumentRequest request,
  ) async {
    openCallCount += 1;
    openSourceKinds.add(request.source.kind);
    final kind = _kindFromRequest(request);
    final title = _titleFromRequest(request, kind);
    return platform.OpenDocumentOpened(
      platform.OpenDocumentSuccess(
        documentId: request.source.kind == platform.DocumentSourceKind.path
            ? 'picked-${kind.name}'
            : 'fixture-${kind.name}',
        kind: kind,
        title: title,
        pageCount: 1,
        capabilities: const platform.DocumentCapabilities(
          search: true,
          textSelection: true,
          passwordProtected: false,
        ),
      ),
    );
  }

  @override
  Future<platform.GetPageRenderModelResult> getPageRenderModel(
    platform.GetPageRenderModelRequest request,
  ) async {
    pageCallCount += 1;
    return platform.GetPageRenderModelSuccess(
      platform.PageRenderModel.fromJson({
        'pageIndex': 0,
        'width': 200.0,
        'height': 300.0,
        'nodes': [
          {
            'type': 'text',
            'text': 'Rendered fixture page',
            'bounds': {'x': 24.0, 'y': 32.0, 'width': 120.0, 'height': 18.0},
            'style': {
              'fontFamily': 'Calibri',
              'fontSize': 12.0,
              'bold': false,
              'italic': false,
              'colorHex': '#000000',
            },
            'range': {'start': 0, 'end': 21},
          },
        ],
        'selectionAnchors': const [],
      }),
    );
  }

  platform.DocumentKind _kindFromRequest(platform.OpenDocumentRequest request) {
    switch (request.source.kind) {
      case platform.DocumentSourceKind.path:
        final value = request.source.value.toLowerCase();
        if (value.endsWith('.pptx')) {
          return platform.DocumentKind.pptx;
        }
        if (value.endsWith('.xlsx')) {
          return platform.DocumentKind.xlsx;
        }
        return platform.DocumentKind.docx;
      case platform.DocumentSourceKind.bytesBase64:
        final marker = utf8.decode(
          base64Decode(request.source.value),
          allowMalformed: true,
        );
        if (marker.contains('assets/fixtures/pptx/')) {
          return platform.DocumentKind.pptx;
        }
        if (marker.contains('assets/fixtures/xlsx/')) {
          return platform.DocumentKind.xlsx;
        }
        return platform.DocumentKind.docx;
    }
  }

  String _titleFromRequest(
    platform.OpenDocumentRequest request,
    platform.DocumentKind kind,
  ) {
    switch (request.source.kind) {
      case platform.DocumentSourceKind.path:
        final segments = request.source.value.split('/');
        return segments.isEmpty ? 'picked.${kind.name}' : segments.last;
      case platform.DocumentSourceKind.bytesBase64:
        final marker = utf8.decode(
          base64Decode(request.source.value),
          allowMalformed: true,
        );
        final segments = marker.split('/');
        return segments.isEmpty
            ? switch (kind) {
                platform.DocumentKind.docx => 'fixture.docx',
                platform.DocumentKind.pptx => 'fixture.pptx',
                platform.DocumentKind.xlsx => 'fixture.xlsx',
              }
            : segments.last;
    }
  }
}

class _FakeAssetBundle extends CachingAssetBundle {
  @override
  Future<ByteData> load(String key) async {
    final bytes = Uint8List.fromList(utf8.encode(key));
    return ByteData.view(bytes.buffer);
  }
}
