import 'package:flutter/foundation.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

import '../errors/open_document_error_mapper.dart';
import '../models/document_descriptor.dart';
import '../password/password_prompt_state.dart';

enum ViewerShellStatus {
  idle,
  loading,
  ready,
  passwordPrompt,
  error,
}

class MsViewerController extends ChangeNotifier {
  MsViewerController({
    this.document,
    MsViewerPlatform? platform,
  }) : _platform = platform ?? MsViewerPlatform.instance;

  final MsViewerPlatform _platform;

  DocumentDescriptor? document;
  MsViewerException? error;
  PasswordPromptState passwordPromptState = const PasswordPromptState(
    status: PasswordPromptStatus.idle,
  );
  ViewerShellStatus status = ViewerShellStatus.idle;
  OpenDocumentRequest? _lastRequest;

  void attachDocument(DocumentDescriptor next) {
    document = next;
    error = null;
    passwordPromptState = const PasswordPromptState(
      status: PasswordPromptStatus.idle,
    );
    status = ViewerShellStatus.ready;
    notifyListeners();
  }

  Future<void> openDocument(OpenDocumentRequest request) async {
    _lastRequest = request;
    status = ViewerShellStatus.loading;
    error = null;
    passwordPromptState = const PasswordPromptState(
      status: PasswordPromptStatus.idle,
    );
    notifyListeners();

    final result = await _platform.openDocument(request);
    switch (result) {
      case OpenDocumentOpened(document: final opened):
        document = DocumentDescriptor.fromOpenDocumentSuccess(opened);
        error = null;
        passwordPromptState = const PasswordPromptState(
          status: PasswordPromptStatus.success,
        );
        status = ViewerShellStatus.ready;
      case OpenDocumentFailure(error: final openError):
        document = null;
        final passwordState = openError.toPasswordPromptState(
          passwordPromptState,
        );
        if (passwordState != null) {
          passwordPromptState = passwordState;
          error = null;
          status = ViewerShellStatus.passwordPrompt;
        } else {
          passwordPromptState = const PasswordPromptState(
            status: PasswordPromptStatus.idle,
          );
          error = openError.toViewerException();
          status = ViewerShellStatus.error;
        }
    }

    notifyListeners();
  }

  Future<void> submitPassword(String password) async {
    final lastRequest = _lastRequest;
    if (lastRequest == null) {
      return;
    }

    await openDocument(
      OpenDocumentRequest(
        source: lastRequest.source,
        options: OpenOptions(
          password: password,
          preferLazyLoading: lastRequest.options.preferLazyLoading,
        ),
      ),
    );
  }
}
