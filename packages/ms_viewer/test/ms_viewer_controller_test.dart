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

  test('controller routes password required failure to password prompt', () async {
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
  });

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
}

class _FakeMsViewerPlatform extends platform.MsViewerPlatform
    with MockPlatformInterfaceMixin {
  _FakeMsViewerPlatform({
    required this.onOpen,
    this.onGetPage,
  });

  final Future<platform.OpenDocumentResult> Function(
    platform.OpenDocumentRequest request,
  ) onOpen;
  final Future<platform.GetPageRenderModelResult> Function(
    platform.GetPageRenderModelRequest request,
  )? onGetPage;

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
