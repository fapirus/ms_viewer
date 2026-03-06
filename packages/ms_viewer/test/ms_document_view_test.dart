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
