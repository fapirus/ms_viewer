import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer/ms_viewer.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart'
    as platform;
import 'package:plugin_platform_interface/plugin_platform_interface.dart';

void main() {
  testWidgets('view shows loading state while open is in flight', (tester) async {
    final completer = Completer<platform.OpenDocumentResult>();
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(
        onOpen: (_) => completer.future,
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: MsDocumentView(controller: controller),
        ),
      ),
    );

    unawaited(
      controller.openDocument(
        const platform.OpenDocumentRequest(
          source: platform.OpenDocumentSource.path('/tmp/sample.docx'),
        ),
      ),
    );
    await tester.pump();

    expect(find.byType(CircularProgressIndicator), findsOneWidget);

    completer.complete(
      platform.OpenDocumentOpened(
        const platform.OpenDocumentSuccess(
          documentId: 'doc_001',
          kind: platform.DocumentKind.docx,
          title: 'sample.docx',
          pageCount: 0,
          capabilities: platform.DocumentCapabilities(
            search: false,
            textSelection: false,
            passwordProtected: false,
          ),
        ),
      ),
    );
    await tester.pump();

    expect(find.textContaining('sample.docx'), findsOneWidget);
  });

  testWidgets('view shows password prompt state', (tester) async {
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(
        onOpen: (_) async => const platform.OpenDocumentFailure(
          platform.OpenDocumentError(
            code: platform.ViewerErrorCode.passwordRequired,
            message: 'Password is required.',
          ),
        ),
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: MsDocumentView(controller: controller),
        ),
      ),
    );

    await controller.openDocument(
      const platform.OpenDocumentRequest(
        source: platform.OpenDocumentSource.path('/tmp/locked.docx'),
      ),
    );
    await tester.pump();

    expect(find.text('Password is required.'), findsOneWidget);
    expect(find.text('Open document'), findsOneWidget);
    expect(find.byType(TextField), findsOneWidget);
  });

  testWidgets('page widget smoke test', (tester) async {
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(
        onOpen: (_) async => platform.OpenDocumentOpened(
          const platform.OpenDocumentSuccess(
            documentId: 'doc_001',
            kind: platform.DocumentKind.docx,
            title: 'sample.docx',
            pageCount: 1,
            capabilities: platform.DocumentCapabilities(
              search: true,
              textSelection: true,
              passwordProtected: false,
            ),
          ),
        ),
      ),
    );

    await controller.openDocument(
      const platform.OpenDocumentRequest(
        source: platform.OpenDocumentSource.path('/tmp/sample.docx'),
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: MsDocumentView(
            controller: controller,
            previewPages: [
              platform.PageRenderModel.fromJson({
                'pageIndex': 0,
                'width': 200.0,
                'height': 300.0,
                'nodes': [
                  {
                    'type': 'text',
                    'text': 'Rendered DOCX page',
                    'bounds': {'x': 24.0, 'y': 32.0, 'width': 120.0, 'height': 18.0},
                    'style': {
                      'fontFamily': 'Calibri',
                      'fontSize': 12.0,
                      'bold': false,
                      'italic': false,
                      'colorHex': '#000000',
                    },
                    'range': {'start': 0, 'end': 18},
                  },
                ],
                'selectionAnchors': [],
              }),
            ],
          ),
        ),
      ),
    );
    await tester.pump();

    expect(find.text('sample.docx'), findsOneWidget);
    expect(find.text('1 pages'), findsOneWidget);
    expect(find.byType(DocumentPageView), findsOneWidget);
    expect(
      find.descendant(
        of: find.byType(DocumentPageView),
        matching: find.byType(CustomPaint),
      ),
      findsWidgets,
    );
  });
}

class _FakeMsViewerPlatform extends platform.MsViewerPlatform
    with MockPlatformInterfaceMixin {
  _FakeMsViewerPlatform({required this.onOpen});

  final Future<platform.OpenDocumentResult> Function(
    platform.OpenDocumentRequest request,
  ) onOpen;

  @override
  Future<platform.OpenDocumentResult> openDocument(
    platform.OpenDocumentRequest request,
  ) {
    return onOpen(request);
  }
}
