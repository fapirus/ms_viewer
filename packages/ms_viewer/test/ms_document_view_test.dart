import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer/ms_viewer.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart'
    as platform;
import 'package:plugin_platform_interface/plugin_platform_interface.dart';

void main() {
  testWidgets('view shows loading state while open is in flight', (
    tester,
  ) async {
    final completer = Completer<platform.OpenDocumentResult>();
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(onOpen: (_) => completer.future),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: MsDocumentView(controller: controller)),
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
        home: Scaffold(body: MsDocumentView(controller: controller)),
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
        onGetPage: (_) async => platform.GetPageRenderModelSuccess(
          platform.PageRenderModel.fromJson({
            'pageIndex': 0,
            'width': 200.0,
            'height': 300.0,
            'nodes': [
              {
                'type': 'text',
                'text': 'Rendered DOCX page',
                'bounds': {
                  'x': 24.0,
                  'y': 32.0,
                  'width': 120.0,
                  'height': 18.0,
                },
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
            'selectionAnchors': const [],
          }),
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
                    'bounds': {
                      'x': 24.0,
                      'y': 32.0,
                      'width': 120.0,
                      'height': 18.0,
                    },
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

  testWidgets('view exposes page navigation controls', (tester) async {
    final pageRequests = <platform.GetPageRenderModelRequest>[];
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(
        onOpen: (_) async => platform.OpenDocumentOpened(
          const platform.OpenDocumentSuccess(
            documentId: 'doc_001',
            kind: platform.DocumentKind.docx,
            title: 'sample.docx',
            pageCount: 2,
            capabilities: platform.DocumentCapabilities(
              search: true,
              textSelection: true,
              passwordProtected: false,
            ),
          ),
        ),
        onGetPage: (request) async {
          pageRequests.add(request);
          return _pageModel(
            _pageJson(
              pageIndex: request.pageIndex,
              text: request.pageIndex == 0 ? 'First page' : 'Second page',
            ),
          );
        },
      ),
    );

    await controller.openDocument(
      const platform.OpenDocumentRequest(
        source: platform.OpenDocumentSource.path('/tmp/sample.docx'),
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: MsDocumentView(controller: controller)),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Page 1 / 2'), findsOneWidget);
    expect(find.widgetWithText(OutlinedButton, 'Next'), findsOneWidget);

    await tester.tap(find.widgetWithText(OutlinedButton, 'Next'));
    await tester.pumpAndSettle();

    expect(find.text('Page 2 / 2'), findsOneWidget);
    expect(pageRequests.map((request) => request.pageIndex), [0, 1]);
  });

  testWidgets('view shows page fetch error state', (tester) async {
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
        onGetPage: (_) async => const platform.GetPageRenderModelFailure(
          platform.OpenDocumentError(
            code: platform.ViewerErrorCode.invalidDocument,
            message: 'Failed to load page preview.',
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
        home: Scaffold(body: MsDocumentView(controller: controller)),
      ),
    );
    await tester.pump();

    expect(find.text('sample.docx'), findsOneWidget);
    expect(find.text('Failed to load page preview.'), findsOneWidget);
    expect(find.byType(DocumentPageView), findsNothing);
  });

  testWidgets('view renders xlsx sheet preview from live fetch', (
    tester,
  ) async {
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(
        onOpen: (_) async => platform.OpenDocumentOpened(
          const platform.OpenDocumentSuccess(
            documentId: 'sheet_001',
            kind: platform.DocumentKind.xlsx,
            title: 'budget.xlsx',
            pageCount: 2,
            capabilities: platform.DocumentCapabilities(
              search: true,
              textSelection: true,
              passwordProtected: false,
            ),
          ),
        ),
        onGetPage: (request) async {
          expect(request.sheetWindow, isNotNull);
          expect(request.sheetWindow!.startRow, 1);
          expect(request.sheetWindow!.endRow, 20);
          expect(request.sheetWindow!.startColumn, 1);
          expect(request.sheetWindow!.endColumn, 8);
          return platform.GetPageRenderModelSuccess(
            platform.PageRenderModel.fromJson({
              'pageIndex': request.pageIndex,
              'width': 280.0,
              'height': 160.0,
              'nodes': [
                {
                  'type': 'box',
                  'bounds': {
                    'x': 0.0,
                    'y': 0.0,
                    'width': 140.0,
                    'height': 40.0,
                  },
                  'fillColorHex': '#1F4E78',
                  'strokeColorHex': '#D0D7DE',
                  'strokeWidth': 1.0,
                },
                {
                  'type': 'text',
                  'text': 'Budget',
                  'bounds': {
                    'x': 24.0,
                    'y': 10.0,
                    'width': 80.0,
                    'height': 18.0,
                  },
                  'style': {
                    'fontFamily': 'Calibri',
                    'fontSize': 12.0,
                    'bold': true,
                    'italic': false,
                    'colorHex': '#FFFFFF',
                  },
                  'range': {'start': 0, 'end': 6},
                },
              ],
              'selectionAnchors': [
                {'nodeIndex': 1, 'charIndex': 0, 'x': 24.0, 'y': 10.0},
                {'nodeIndex': 1, 'charIndex': 6, 'x': 72.0, 'y': 10.0},
              ],
            }),
          );
        },
      ),
    );

    await controller.openDocument(
      const platform.OpenDocumentRequest(
        source: platform.OpenDocumentSource.path('/tmp/budget.xlsx'),
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: MsDocumentView(controller: controller),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('budget.xlsx'), findsOneWidget);
    expect(find.text('2 pages'), findsOneWidget);
    expect(find.byType(DocumentPageView), findsOneWidget);
    expect(find.widgetWithText(OutlinedButton, 'Next'), findsOneWidget);
  });

  testWidgets('view shows xlsx sheet fetch error state', (tester) async {
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(
        onOpen: (_) async => platform.OpenDocumentOpened(
          const platform.OpenDocumentSuccess(
            documentId: 'sheet_001',
            kind: platform.DocumentKind.xlsx,
            title: 'budget.xlsx',
            pageCount: 1,
            capabilities: platform.DocumentCapabilities(
              search: true,
              textSelection: true,
              passwordProtected: false,
            ),
          ),
        ),
        onGetPage: (_) async => const platform.GetPageRenderModelFailure(
          platform.OpenDocumentError(
            code: platform.ViewerErrorCode.invalidDocument,
            message: 'Failed to load sheet preview.',
          ),
        ),
      ),
    );

    await controller.openDocument(
      const platform.OpenDocumentRequest(
        source: platform.OpenDocumentSource.path('/tmp/budget.xlsx'),
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: MsDocumentView(controller: controller)),
      ),
    );
    await tester.pump();

    expect(find.text('budget.xlsx'), findsOneWidget);
    expect(find.text('Failed to load sheet preview.'), findsOneWidget);
    expect(find.byType(DocumentPageView), findsNothing);
  });

  testWidgets('xlsx sheet search integration widget test', (tester) async {
    platform.GetSelectionPageRequest? selectionRequest;
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(
        onOpen: (_) async => platform.OpenDocumentOpened(
          const platform.OpenDocumentSuccess(
            documentId: 'sheet_001',
            kind: platform.DocumentKind.xlsx,
            title: 'budget.xlsx',
            pageCount: 2,
            capabilities: platform.DocumentCapabilities(
              search: true,
              textSelection: true,
              passwordProtected: false,
            ),
          ),
        ),
        onGetPage: (_) async =>
            _pageModel(_pageJson(pageIndex: 0, text: 'Budget summary')),
        onSearch: (_) async => const platform.SearchDocumentSuccess([
          platform.SearchMatchModel(
            query: 'status',
            pageIndex: 1,
            start: 0,
            end: 6,
            preview: 'status column on detail sheet',
          ),
        ]),
        onGetSelectionPage: (request) async {
          selectionRequest = request;
          return platform.GetSelectionPageSuccess(
            platform.PageRenderModel.fromJson(
              _pageJson(pageIndex: 1, text: 'status column on detail sheet'),
            ),
          );
        },
      ),
    );

    await controller.openDocument(
      const platform.OpenDocumentRequest(
        source: platform.OpenDocumentSource.path('/tmp/budget.xlsx'),
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: MsDocumentView(controller: controller)),
      ),
    );
    await tester.pump();

    await tester.enterText(find.byType(TextField).first, 'status');
    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();

    expect(find.text('status column on detail sheet'), findsOneWidget);

    await tester.tap(find.text('status column on detail sheet'));
    await tester.pumpAndSettle();

    expect(selectionRequest, isNotNull);
    expect(selectionRequest!.pageIndex, 1);
    expect(controller.currentPageIndex, 1);
    expect(controller.pageHighlights, isNotEmpty);
  });

  testWidgets('xlsx sheet text selection integration widget test', (
    tester,
  ) async {
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(
        onOpen: (_) async => platform.OpenDocumentOpened(
          const platform.OpenDocumentSuccess(
            documentId: 'sheet_001',
            kind: platform.DocumentKind.xlsx,
            title: 'budget.xlsx',
            pageCount: 1,
            capabilities: platform.DocumentCapabilities(
              search: true,
              textSelection: true,
              passwordProtected: false,
            ),
          ),
        ),
        onGetPage: (_) async =>
            _pageModel(_pageJson(pageIndex: 0, text: 'North Region 420')),
      ),
    );

    await controller.openDocument(
      const platform.OpenDocumentRequest(
        source: platform.OpenDocumentSource.path('/tmp/budget.xlsx'),
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: MsDocumentView(controller: controller)),
      ),
    );
    await controller.loadPage(0);
    await tester.pumpAndSettle();

    final pageFinder = find.byType(DocumentPageView);
    expect(pageFinder, findsOneWidget);
    final pageRect = tester.getRect(pageFinder);

    final gesture = await tester.startGesture(
      Offset(
        pageRect.left + pageRect.width * 0.18,
        pageRect.top + pageRect.height * 0.12,
      ),
    );
    await gesture.moveBy(Offset(pageRect.width * 0.4, pageRect.height * 0.05));
    await tester.pump();

    expect(controller.pageHighlights, isNotEmpty);
    expect(find.byType(Positioned), findsWidgets);
  });

  testWidgets('search action integration widget test', (tester) async {
    platform.GetSelectionPageRequest? selectionRequest;
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(
        onOpen: (_) async => platform.OpenDocumentOpened(
          const platform.OpenDocumentSuccess(
            documentId: 'doc_001',
            kind: platform.DocumentKind.docx,
            title: 'sample.docx',
            pageCount: 3,
            capabilities: platform.DocumentCapabilities(
              search: true,
              textSelection: true,
              passwordProtected: false,
            ),
          ),
        ),
        onGetPage: (_) async =>
            _pageModel(_pageJson(pageIndex: 0, text: 'Page zero preview')),
        onSearch: (_) async => const platform.SearchDocumentSuccess([
          platform.SearchMatchModel(
            query: 'needle',
            pageIndex: 2,
            start: 0,
            end: 6,
            preview: 'needle on third page',
          ),
        ]),
        onGetSelectionPage: (request) async {
          selectionRequest = request;
          return platform.GetSelectionPageSuccess(
            platform.PageRenderModel.fromJson(
              _pageJson(pageIndex: 2, text: 'needle on third page'),
            ),
          );
        },
      ),
    );

    await controller.openDocument(
      const platform.OpenDocumentRequest(
        source: platform.OpenDocumentSource.path('/tmp/sample.docx'),
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: MsDocumentView(controller: controller)),
      ),
    );
    await tester.pump();

    await tester.enterText(find.byType(TextField).first, 'needle');
    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();

    expect(find.text('needle on third page'), findsOneWidget);

    await tester.tap(find.text('needle on third page'));
    await tester.pumpAndSettle();

    expect(selectionRequest, isNotNull);
    expect(selectionRequest!.pageIndex, 2);
    expect(controller.currentPageIndex, 2);
    expect(controller.pageHighlights, isNotEmpty);
  });

  testWidgets('selection overlay integration widget test', (tester) async {
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
        onGetPage: (_) async =>
            _pageModel(_pageJson(pageIndex: 0, text: 'Selectable text')),
      ),
    );

    await controller.openDocument(
      const platform.OpenDocumentRequest(
        source: platform.OpenDocumentSource.path('/tmp/sample.docx'),
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: MsDocumentView(controller: controller)),
      ),
    );
    await controller.loadPage(0);
    await tester.pumpAndSettle();

    final pageFinder = find.byType(DocumentPageView);
    expect(pageFinder, findsOneWidget);
    final pageRect = tester.getRect(pageFinder);

    final gesture = await tester.startGesture(
      Offset(
        pageRect.left + pageRect.width * 0.18,
        pageRect.top + pageRect.height * 0.12,
      ),
    );
    await gesture.moveBy(Offset(pageRect.width * 0.4, pageRect.height * 0.05));
    await tester.pump();

    expect(controller.pageHighlights, isNotEmpty);
    expect(find.byType(Positioned), findsWidgets);
  });

  testWidgets('view renders pptx slide preview and navigation shell', (
    tester,
  ) async {
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(
        onOpen: (_) async => platform.OpenDocumentOpened(
          const platform.OpenDocumentSuccess(
            documentId: 'ppt_001',
            kind: platform.DocumentKind.pptx,
            title: 'deck.pptx',
            pageCount: 2,
            capabilities: platform.DocumentCapabilities(
              search: true,
              textSelection: true,
              passwordProtected: false,
            ),
          ),
        ),
        onGetPage: (request) async => platform.GetPageRenderModelSuccess(
          platform.PageRenderModel.fromJson({
            'pageIndex': request.pageIndex,
            'width': 720.0,
            'height': 540.0,
            'nodes': [
              {
                'type': 'box',
                'bounds': {
                  'x': 40.0,
                  'y': 60.0,
                  'width': 180.0,
                  'height': 72.0,
                },
                'fillColorHex': '#FFAA00',
                'strokeColorHex': '#333333',
                'strokeWidth': 1.0,
              },
              {
                'type': 'text',
                'text': request.pageIndex == 0
                    ? 'Quarterly Results'
                    : 'Revenue Forecast',
                'bounds': {
                  'x': 66.0,
                  'y': 24.0,
                  'width': 220.0,
                  'height': 28.0,
                },
                'style': {
                  'fontFamily': 'Aptos',
                  'fontSize': 24.0,
                  'bold': true,
                  'italic': false,
                  'colorHex': '#000000',
                },
                'range': {
                  'start': 0,
                  'end': request.pageIndex == 0 ? 17 : 16,
                },
              },
              {
                'type': 'image',
                'resourceId': 'ppt/media/image1.png',
                'description': 'Fixture image',
                'contentType': 'image/png',
                'dataBase64':
                    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+lm2cAAAAASUVORK5CYII=',
                'bounds': {
                  'x': 300.0,
                  'y': 120.0,
                  'width': 100.0,
                  'height': 75.0,
                },
              },
            ],
            'selectionAnchors': const [
              {'nodeIndex': 1, 'charIndex': 0, 'x': 66.0, 'y': 24.0},
            ],
          }),
        ),
      ),
    );

    await controller.openDocument(
      const platform.OpenDocumentRequest(
        source: platform.OpenDocumentSource.path('/tmp/deck.pptx'),
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: MsDocumentView(controller: controller)),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('deck.pptx'), findsOneWidget);
    expect(find.text('2 pages'), findsOneWidget);
    expect(find.text('Page 1 / 2'), findsOneWidget);
    expect(find.byType(DocumentPageView), findsOneWidget);
    expect(find.byType(Image), findsOneWidget);

    await tester.tap(find.widgetWithText(OutlinedButton, 'Next'));
    await tester.pumpAndSettle();

    expect(find.text('Page 2 / 2'), findsOneWidget);
  });

  testWidgets('view shows pptx slide fetch error state', (tester) async {
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(
        onOpen: (_) async => platform.OpenDocumentOpened(
          const platform.OpenDocumentSuccess(
            documentId: 'ppt_001',
            kind: platform.DocumentKind.pptx,
            title: 'deck.pptx',
            pageCount: 1,
            capabilities: platform.DocumentCapabilities(
              search: true,
              textSelection: true,
              passwordProtected: false,
            ),
          ),
        ),
        onGetPage: (_) async => const platform.GetPageRenderModelFailure(
          platform.OpenDocumentError(
            code: platform.ViewerErrorCode.invalidDocument,
            message: 'Failed to load slide preview.',
          ),
        ),
      ),
    );

    await controller.openDocument(
      const platform.OpenDocumentRequest(
        source: platform.OpenDocumentSource.path('/tmp/deck.pptx'),
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: MsDocumentView(controller: controller)),
      ),
    );
    await tester.pump();

    expect(find.text('deck.pptx'), findsOneWidget);
    expect(find.text('Failed to load slide preview.'), findsOneWidget);
    expect(find.byType(DocumentPageView), findsNothing);
  });

  testWidgets('pptx slide search integration widget test', (tester) async {
    platform.GetSelectionPageRequest? selectionRequest;
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(
        onOpen: (_) async => platform.OpenDocumentOpened(
          const platform.OpenDocumentSuccess(
            documentId: 'ppt_001',
            kind: platform.DocumentKind.pptx,
            title: 'deck.pptx',
            pageCount: 3,
            capabilities: platform.DocumentCapabilities(
              search: true,
              textSelection: true,
              passwordProtected: false,
            ),
          ),
        ),
        onGetPage: (_) async => _pageModel(
          _pageJson(pageIndex: 0, text: 'Agenda overview'),
        ),
        onSearch: (_) async => const platform.SearchDocumentSuccess([
          platform.SearchMatchModel(
            query: 'forecast',
            pageIndex: 2,
            start: 0,
            end: 8,
            preview: 'forecast on closing slide',
          ),
        ]),
        onGetSelectionPage: (request) async {
          selectionRequest = request;
          return platform.GetSelectionPageSuccess(
            platform.PageRenderModel.fromJson(
              _pageJson(pageIndex: 2, text: 'forecast on closing slide'),
            ),
          );
        },
      ),
    );

    await controller.openDocument(
      const platform.OpenDocumentRequest(
        source: platform.OpenDocumentSource.path('/tmp/deck.pptx'),
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: MsDocumentView(controller: controller)),
      ),
    );
    await tester.pump();

    await tester.enterText(find.byType(TextField).first, 'forecast');
    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();

    expect(find.text('forecast on closing slide'), findsOneWidget);

    await tester.tap(find.text('forecast on closing slide'));
    await tester.pumpAndSettle();

    expect(selectionRequest, isNotNull);
    expect(selectionRequest!.pageIndex, 2);
    expect(controller.currentPageIndex, 2);
    expect(controller.pageHighlights, isNotEmpty);
  });

  testWidgets('pptx slide selection integration widget test', (tester) async {
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(
        onOpen: (_) async => platform.OpenDocumentOpened(
          const platform.OpenDocumentSuccess(
            documentId: 'ppt_001',
            kind: platform.DocumentKind.pptx,
            title: 'deck.pptx',
            pageCount: 1,
            capabilities: platform.DocumentCapabilities(
              search: true,
              textSelection: true,
              passwordProtected: false,
            ),
          ),
        ),
        onGetPage: (_) async => platform.GetPageRenderModelSuccess(
          platform.PageRenderModel.fromJson({
            'pageIndex': 0,
            'width': 720.0,
            'height': 540.0,
            'nodes': [
              {
                'type': 'text',
                'text': 'Quarterly Results',
                'bounds': {
                  'x': 66.0,
                  'y': 24.0,
                  'width': 220.0,
                  'height': 28.0,
                },
                'style': {
                  'fontFamily': 'Aptos',
                  'fontSize': 24.0,
                  'bold': true,
                  'italic': false,
                  'colorHex': '#000000',
                },
                'range': {'start': 0, 'end': 17},
              },
            ],
            'selectionAnchors': const [
              {'nodeIndex': 0, 'charIndex': 0, 'x': 66.0, 'y': 24.0},
              {'nodeIndex': 0, 'charIndex': 17, 'x': 286.0, 'y': 24.0},
            ],
          }),
        ),
      ),
    );

    await controller.openDocument(
      const platform.OpenDocumentRequest(
        source: platform.OpenDocumentSource.path('/tmp/deck.pptx'),
      ),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: MsDocumentView(controller: controller)),
      ),
    );
    await tester.pumpAndSettle();

    final pageFinder = find.byType(DocumentPageView);
    expect(pageFinder, findsOneWidget);
    final pageRect = tester.getRect(pageFinder);
    final start = Offset(
      pageRect.left + pageRect.width * (66.0 / 720.0),
      pageRect.top + pageRect.height * (24.0 / 540.0),
    );
    final delta = Offset(
      pageRect.width * (120.0 / 720.0),
      pageRect.height * (18.0 / 540.0),
    );

    final gesture = await tester.startGesture(start);
    await gesture.moveBy(delta);
    await tester.pump();

    expect(controller.pageHighlights, isNotEmpty);
    expect(find.byType(Positioned), findsWidgets);
  });
}

class _FakeMsViewerPlatform extends platform.MsViewerPlatform
    with MockPlatformInterfaceMixin {
  _FakeMsViewerPlatform({
    required this.onOpen,
    this.onGetPage,
    this.onSearch,
    this.onGetSelectionPage,
  });

  final Future<platform.OpenDocumentResult> Function(
    platform.OpenDocumentRequest request,
  )
  onOpen;
  final Future<platform.GetPageRenderModelResult> Function(
    platform.GetPageRenderModelRequest request,
  )?
  onGetPage;
  final Future<platform.SearchDocumentResult> Function(
    platform.SearchDocumentRequest request,
  )?
  onSearch;
  final Future<platform.GetSelectionPageResult> Function(
    platform.GetSelectionPageRequest request,
  )?
  onGetSelectionPage;

  @override
  Future<platform.OpenDocumentResult> openDocument(
    platform.OpenDocumentRequest request,
  ) {
    return onOpen(request);
  }

  @override
  Future<platform.GetPageRenderModelResult> getPageRenderModel(
    platform.GetPageRenderModelRequest request,
  ) {
    final onGetPage = this.onGetPage;
    if (onGetPage == null) {
      return Future.value(
        const platform.GetPageRenderModelFailure(
          platform.OpenDocumentError(
            code: platform.ViewerErrorCode.notImplemented,
            message: 'getPageRenderModel is not implemented.',
          ),
        ),
      );
    }
    return onGetPage(request);
  }

  @override
  Future<platform.SearchDocumentResult> searchDocument(
    platform.SearchDocumentRequest request,
  ) {
    final onSearch = this.onSearch;
    if (onSearch == null) {
      return Future.value(
        const platform.SearchDocumentFailure(
          platform.OpenDocumentError(
            code: platform.ViewerErrorCode.notImplemented,
            message: 'searchDocument is not implemented.',
          ),
        ),
      );
    }
    return onSearch(request);
  }

  @override
  Future<platform.GetSelectionPageResult> getSelectionPage(
    platform.GetSelectionPageRequest request,
  ) {
    final onGetSelectionPage = this.onGetSelectionPage;
    if (onGetSelectionPage == null) {
      return Future.value(
        const platform.GetSelectionPageFailure(
          platform.OpenDocumentError(
            code: platform.ViewerErrorCode.notImplemented,
            message: 'getSelectionPage is not implemented.',
          ),
        ),
      );
    }
    return onGetSelectionPage(request);
  }
}

platform.GetPageRenderModelSuccess _pageModel(Map<String, Object?> json) {
  return platform.GetPageRenderModelSuccess(
    platform.PageRenderModel.fromJson(json),
  );
}

Map<String, Object?> _pageJson({required int pageIndex, required String text}) {
  return {
    'pageIndex': pageIndex,
    'width': 200.0,
    'height': 300.0,
    'nodes': [
      {
        'type': 'text',
        'text': text,
        'bounds': {'x': 24.0, 'y': 32.0, 'width': 120.0, 'height': 18.0},
        'style': {
          'fontFamily': 'Calibri',
          'fontSize': 12.0,
          'bold': false,
          'italic': false,
          'colorHex': '#000000',
        },
        'range': {'start': 0, 'end': text.length},
      },
    ],
    'selectionAnchors': const [
      {'nodeIndex': 0, 'charIndex': 0, 'x': 24.0, 'y': 32.0},
    ],
  };
}
