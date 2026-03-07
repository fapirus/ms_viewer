import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_demo/main.dart';
import 'package:flutter/services.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart'
    as platform;

void main() {
  testWidgets('demo app renders fixture browser shell', (tester) async {
    final platform = _FakeDemoPlatform();
    await tester.pumpWidget(
      DemoApp(
        platform: platform,
        assetBundle: _FakeAssetBundle(),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('MS Viewer Demo'), findsOneWidget);
    expect(find.text('Fixtures'), findsOneWidget);
    expect(find.text('Open File'), findsOneWidget);
    expect(find.text('docx_plain_text.docx'), findsWidgets);
    expect(find.text('Bundled fixture'), findsWidgets);
    expect(platform.openCallCount, 1);
    expect(platform.pageCallCount, 1);
  });
}

class _FakeDemoPlatform extends platform.MsViewerPlatform {
  int openCallCount = 0;
  int pageCallCount = 0;

  @override
  Future<platform.OpenDocumentResult> openDocument(
    platform.OpenDocumentRequest request,
  ) async {
    openCallCount += 1;
    expect(request.source.kind, platform.DocumentSourceKind.bytesBase64);
    return platform.OpenDocumentOpened(
      const platform.OpenDocumentSuccess(
        documentId: 'fixture-docx',
        kind: platform.DocumentKind.docx,
        title: 'docx_plain_text.docx',
        pageCount: 1,
        capabilities: platform.DocumentCapabilities(
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
}

class _FakeAssetBundle extends CachingAssetBundle {
  @override
  Future<ByteData> load(String key) async {
    final bytes = Uint8List.fromList(const [0x50, 0x4B, 0x03, 0x04]);
    return ByteData.view(bytes.buffer);
  }
}
