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
            pageCount: 0,
            capabilities: platform.DocumentCapabilities(
              search: false,
              textSelection: false,
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

    expect(controller.status, ViewerShellStatus.ready);
    expect(controller.document, isNotNull);
    expect(controller.document!.title, 'sample.docx');
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
              pageCount: 0,
              capabilities: platform.DocumentCapabilities(
                search: false,
                textSelection: false,
                passwordProtected: false,
              ),
            ),
          );
        },
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
