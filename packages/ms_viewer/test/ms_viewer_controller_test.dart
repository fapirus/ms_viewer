import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer/ms_viewer.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart'
    as platform;
import 'package:plugin_platform_interface/plugin_platform_interface.dart';

void main() {
  test('controller opens document and attaches descriptor', () async {
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(
        onOpen: (_) async => platform.OpenDocumentOpened(
          const platform.OpenDocumentSuccess(
            documentId: 'doc_001',
            kind: platform.DocumentKind.docx,
            title: 'sample.docx',
            pageCount: 1,
            capabilities: platform.DocumentCapabilities(
              search: false,
              textSelection: false,
              passwordProtected: false,
            ),
          ),
        ),
        onGetPage: (_) async => platform.GetPageRenderModelSuccess(
          platform.PageRenderModel.fromJson({
            'pageIndex': 0,
            'width': 595.0,
            'height': 842.0,
            'nodes': const [],
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

    expect(controller.status, ViewerShellStatus.ready);
    expect(controller.document, isNotNull);
    expect(controller.document!.title, 'sample.docx');
    expect(controller.pageStatus, ViewerPageStatus.ready);
    expect(controller.currentPage, isNotNull);
  });

  test(
    'controller routes password required failure to password prompt',
    () async {
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

      await controller.openDocument(
        const platform.OpenDocumentRequest(
          source: platform.OpenDocumentSource.path('/tmp/locked.docx'),
        ),
      );

      expect(controller.status, ViewerShellStatus.passwordPrompt);
      expect(
        controller.passwordPromptState.status,
        PasswordPromptStatus.awaitingPassword,
      );
    },
  );

  test('submitPassword replays last request with password', () async {
    platform.OpenDocumentRequest? capturedRetry;
    var callCount = 0;
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(
        onOpen: (request) async {
          callCount += 1;
          if (callCount == 1) {
            return const platform.OpenDocumentFailure(
              platform.OpenDocumentError(
                code: platform.ViewerErrorCode.passwordRequired,
                message: 'Password is required.',
              ),
            );
          }
          capturedRetry = request;
          return platform.OpenDocumentOpened(
            const platform.OpenDocumentSuccess(
              documentId: 'doc_locked',
              kind: platform.DocumentKind.docx,
              title: 'locked.docx',
              pageCount: 1,
              capabilities: platform.DocumentCapabilities(
                search: false,
                textSelection: false,
                passwordProtected: false,
              ),
            ),
          );
        },
        onGetPage: (_) async => platform.GetPageRenderModelSuccess(
          platform.PageRenderModel.fromJson({
            'pageIndex': 0,
            'width': 595.0,
            'height': 842.0,
            'nodes': const [],
            'selectionAnchors': const [],
          }),
        ),
      ),
    );

    await controller.openDocument(
      const platform.OpenDocumentRequest(
        source: platform.OpenDocumentSource.path('/tmp/locked.docx'),
      ),
    );
    await controller.submitPassword('secret');

    expect(capturedRetry, isNotNull);
    expect(capturedRetry!.options.password, 'secret');
    expect(controller.status, ViewerShellStatus.ready);
  });

  test('controller fetches first page after opening docx', () async {
    platform.GetPageRenderModelRequest? capturedPageRequest;
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
        onGetPage: (request) async {
          capturedPageRequest = request;
          return platform.GetPageRenderModelSuccess(
            platform.PageRenderModel.fromJson({
              'pageIndex': 0,
              'width': 595.0,
              'height': 842.0,
              'nodes': const [],
              'selectionAnchors': const [],
            }),
          );
        },
      ),
    );

    await controller.openDocument(
      const platform.OpenDocumentRequest(
        source: platform.OpenDocumentSource.path('/tmp/sample.docx'),
      ),
    );

    expect(capturedPageRequest, isNotNull);
    expect(capturedPageRequest!.documentId, 'doc_001');
    expect(capturedPageRequest!.pageIndex, 0);
    expect(controller.pageStatus, ViewerPageStatus.ready);
    expect(controller.currentPageIndex, 0);
  });

  test('controller fetches first slide after opening pptx', () async {
    platform.GetPageRenderModelRequest? capturedPageRequest;
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
        onGetPage: (request) async {
          capturedPageRequest = request;
          return platform.GetPageRenderModelSuccess(
            platform.PageRenderModel.fromJson({
              'pageIndex': 0,
              'width': 720.0,
              'height': 540.0,
              'nodes': const [],
              'selectionAnchors': const [],
            }),
          );
        },
      ),
    );

    await controller.openDocument(
      const platform.OpenDocumentRequest(
        source: platform.OpenDocumentSource.path('/tmp/deck.pptx'),
      ),
    );

    expect(capturedPageRequest, isNotNull);
    expect(capturedPageRequest!.documentId, 'ppt_001');
    expect(capturedPageRequest!.pageIndex, 0);
    expect(controller.pageStatus, ViewerPageStatus.ready);
    expect(controller.currentPageIndex, 0);
  });

  test('controller fetches first sheet window after opening xlsx', () async {
    platform.GetPageRenderModelRequest? capturedPageRequest;
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(
        onOpen: (_) async => platform.OpenDocumentOpened(
          const platform.OpenDocumentSuccess(
            documentId: 'xlsx_001',
            kind: platform.DocumentKind.xlsx,
            title: 'budget.xlsx',
            pageCount: 2,
            capabilities: platform.DocumentCapabilities(
              search: true,
              textSelection: true,
              passwordProtected: false,
            ),
            sheetTabs: [
              platform.SheetTabModel(pageIndex: 0, title: 'Summary'),
              platform.SheetTabModel(pageIndex: 1, title: 'Detail'),
            ],
            activePageIndex: 1,
          ),
        ),
        onGetPage: (request) async {
          capturedPageRequest = request;
          return platform.GetPageRenderModelSuccess(
            platform.PageRenderModel.fromJson({
              'pageIndex': 1,
              'width': 640.0,
              'height': 360.0,
              'nodes': const [],
              'selectionAnchors': const [],
              'sheetViewport': {
                'window': {
                  'startRow': request.sheetWindow!.startRow,
                  'endRow': request.sheetWindow!.endRow,
                  'startColumn': request.sheetWindow!.startColumn,
                  'endColumn': request.sheetWindow!.endColumn,
                },
                'effectiveBounds': {
                  'startRow': 1,
                  'endRow': 120,
                  'startColumn': 1,
                  'endColumn': 24,
                },
                'visibleRows': [1, 2, 3],
                'visibleColumns': [1, 2, 3],
              },
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

    expect(capturedPageRequest, isNotNull);
    expect(capturedPageRequest!.documentId, 'xlsx_001');
    expect(capturedPageRequest!.pageIndex, 1);
    expect(capturedPageRequest!.sheetWindow, isNotNull);
    expect(capturedPageRequest!.sheetWindow!.startRow, 1);
    expect(capturedPageRequest!.sheetWindow!.endRow, 48);
    expect(capturedPageRequest!.sheetWindow!.startColumn, 1);
    expect(capturedPageRequest!.sheetWindow!.endColumn, 16);
    expect(controller.pageStatus, ViewerPageStatus.ready);
    expect(controller.currentPageIndex, 1);
  });

  test('controller reloads xlsx with requested sheet window', () async {
    final requests = <platform.GetPageRenderModelRequest>[];
    final controller = MsViewerController(
      platform: _FakeMsViewerPlatform(
        onOpen: (_) async => platform.OpenDocumentOpened(
          const platform.OpenDocumentSuccess(
            documentId: 'xlsx_001',
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
          requests.add(request);
          return platform.GetPageRenderModelSuccess(
            platform.PageRenderModel.fromJson({
              'pageIndex': 0,
              'width': 640.0,
              'height': 360.0,
              'nodes': const [],
              'selectionAnchors': const [],
              'sheetViewport': {
                'window': {
                  'startRow': request.sheetWindow!.startRow,
                  'endRow': request.sheetWindow!.endRow,
                  'startColumn': request.sheetWindow!.startColumn,
                  'endColumn': request.sheetWindow!.endColumn,
                },
                'effectiveBounds': {
                  'startRow': 1,
                  'endRow': 120,
                  'startColumn': 1,
                  'endColumn': 24,
                },
                'visibleRows': [
                  for (var row = 0; row < 3; row++)
                    request.sheetWindow!.startRow + row,
                ],
                'visibleColumns': [
                  for (var column = 0; column < 3; column++)
                    request.sheetWindow!.startColumn + column,
                ],
              },
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
    await controller.loadSheetWindow(
      const platform.SheetWindow(
        startRow: 55,
        endRow: 78,
        startColumn: 17,
        endColumn: 20,
      ),
    );

    expect(requests, hasLength(2));
    expect(requests.last.sheetWindow, isNotNull);
    expect(requests.last.sheetWindow!.startRow, 49);
    expect(requests.last.sheetWindow!.endRow, 96);
    expect(requests.last.sheetWindow!.startColumn, 17);
    expect(requests.last.sheetWindow!.endColumn, 32);
    expect(controller.currentSheetWindow.startRow, 49);
    expect(controller.currentSheetWindow.endColumn, 32);
    expect(controller.pageStatus, ViewerPageStatus.ready);
    expect(controller.currentPage, isNotNull);
  });

  test(
    'controller keeps current xlsx page visible while loading next sheet window',
    () async {
      final secondWindowCompleter =
          Completer<platform.GetPageRenderModelResult>();
      var requestCount = 0;
      final controller = MsViewerController(
        platform: _FakeMsViewerPlatform(
          onOpen: (_) async => platform.OpenDocumentOpened(
            const platform.OpenDocumentSuccess(
              documentId: 'xlsx_001',
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
          onGetPage: (request) {
            requestCount += 1;
            if (requestCount == 1) {
              return Future.value(
                platform.GetPageRenderModelSuccess(
                  platform.PageRenderModel.fromJson({
                    'pageIndex': 0,
                    'width': 640.0,
                    'height': 360.0,
                    'nodes': const [],
                    'selectionAnchors': const [],
                    'sheetViewport': {
                      'window': {
                        'startRow': 1,
                        'endRow': 48,
                        'startColumn': 1,
                        'endColumn': 16,
                      },
                      'effectiveBounds': {
                        'startRow': 1,
                        'endRow': 120,
                        'startColumn': 1,
                        'endColumn': 24,
                      },
                      'visibleRows': [1, 2, 3],
                      'visibleColumns': [1, 2, 3],
                    },
                  }),
                ),
              );
            }
            return secondWindowCompleter.future;
          },
        ),
      );

      await controller.openDocument(
        const platform.OpenDocumentRequest(
          source: platform.OpenDocumentSource.path('/tmp/budget.xlsx'),
        ),
      );

      final originalPage = controller.currentPage;
      final loadFuture = controller.loadSheetWindow(
        const platform.SheetWindow(
          startRow: 55,
          endRow: 78,
          startColumn: 17,
          endColumn: 20,
        ),
      );

      expect(controller.currentPage, same(originalPage));
      expect(controller.isSheetWindowLoading, isTrue);
      expect(controller.pageStatus, ViewerPageStatus.ready);

      secondWindowCompleter.complete(
        platform.GetPageRenderModelSuccess(
          platform.PageRenderModel.fromJson({
            'pageIndex': 0,
            'width': 640.0,
            'height': 360.0,
            'nodes': const [],
            'selectionAnchors': const [],
            'sheetViewport': {
              'window': {
                'startRow': 49,
                'endRow': 96,
                'startColumn': 17,
                'endColumn': 32,
              },
              'effectiveBounds': {
                'startRow': 1,
                'endRow': 120,
                'startColumn': 1,
                'endColumn': 24,
              },
              'visibleRows': [49, 50, 51],
              'visibleColumns': [17, 18, 19],
            },
          }),
        ),
      );
      await loadFuture;

      expect(controller.isSheetWindowLoading, isFalse);
      expect(controller.currentPage, isNot(same(originalPage)));
      expect(controller.currentSheetWindow.startRow, 49);
    },
  );

  test('controller navigates to next page when requested', () async {
    final requests = <platform.GetPageRenderModelRequest>[];
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
        onGetPage: (request) async {
          requests.add(request);
          return platform.GetPageRenderModelSuccess(
            platform.PageRenderModel.fromJson({
              'pageIndex': request.pageIndex,
              'width': 595.0,
              'height': 842.0,
              'nodes': const [],
              'selectionAnchors': const [],
            }),
          );
        },
      ),
    );

    await controller.openDocument(
      const platform.OpenDocumentRequest(
        source: platform.OpenDocumentSource.path('/tmp/sample.docx'),
      ),
    );
    await controller.goToNextPage();

    expect(requests.map((request) => request.pageIndex), [0, 1]);
    expect(controller.currentPageIndex, 1);
    expect(controller.canGoToPreviousPage, isTrue);
    expect(controller.canGoToNextPage, isTrue);
  });
}

class _FakeMsViewerPlatform extends platform.MsViewerPlatform
    with MockPlatformInterfaceMixin {
  _FakeMsViewerPlatform({required this.onOpen, this.onGetPage});

  final Future<platform.OpenDocumentResult> Function(
    platform.OpenDocumentRequest request,
  )
  onOpen;
  final Future<platform.GetPageRenderModelResult> Function(
    platform.GetPageRenderModelRequest request,
  )?
  onGetPage;

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
}
