import 'package:flutter/foundation.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart'
    as viewer_platform;

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

enum ViewerPageStatus {
  idle,
  loading,
  ready,
  error,
}

class MsViewerController extends ChangeNotifier {
  MsViewerController({
    this.document,
    viewer_platform.MsViewerPlatform? platform,
  }) : _platform = platform ?? viewer_platform.MsViewerPlatform.instance;

  final viewer_platform.MsViewerPlatform _platform;

  DocumentDescriptor? document;
  MsViewerException? error;
  MsViewerException? pageError;
  viewer_platform.PageRenderModel? currentPage;
  int? currentPageIndex;
  PasswordPromptState passwordPromptState = const PasswordPromptState(
    status: PasswordPromptStatus.idle,
  );
  ViewerShellStatus status = ViewerShellStatus.idle;
  ViewerPageStatus pageStatus = ViewerPageStatus.idle;
  viewer_platform.OpenDocumentRequest? _lastRequest;

  void attachDocument(DocumentDescriptor next) {
    document = next;
    error = null;
    currentPage = null;
    currentPageIndex = null;
    pageError = null;
    pageStatus = ViewerPageStatus.idle;
    passwordPromptState = const PasswordPromptState(
      status: PasswordPromptStatus.idle,
    );
    status = ViewerShellStatus.ready;
    notifyListeners();
  }

  Future<void> openDocument(viewer_platform.OpenDocumentRequest request) async {
    _lastRequest = request;
    status = ViewerShellStatus.loading;
    error = null;
    currentPage = null;
    currentPageIndex = null;
    pageError = null;
    pageStatus = ViewerPageStatus.idle;
    passwordPromptState = const PasswordPromptState(
      status: PasswordPromptStatus.idle,
    );
    notifyListeners();

    final result = await _platform.openDocument(request);
    switch (result) {
      case viewer_platform.OpenDocumentOpened(document: final opened):
        document = DocumentDescriptor.fromOpenDocumentSuccess(opened);
        error = null;
        passwordPromptState = const PasswordPromptState(
          status: PasswordPromptStatus.success,
        );
        status = ViewerShellStatus.ready;
        if (opened.kind == viewer_platform.DocumentKind.docx &&
            opened.pageCount > 0) {
          await loadPage(0);
        }
      case viewer_platform.OpenDocumentFailure(error: final openError):
        document = null;
        currentPage = null;
        currentPageIndex = null;
        pageError = null;
        pageStatus = ViewerPageStatus.idle;
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

  Future<void> loadPage(int pageIndex) async {
    final lastRequest = _lastRequest;
    final descriptor = document;
    if (lastRequest == null || descriptor == null) {
      return;
    }

    pageStatus = ViewerPageStatus.loading;
    currentPage = null;
    currentPageIndex = pageIndex;
    pageError = null;
    notifyListeners();

    final result = await _platform.getPageRenderModel(
      viewer_platform.GetPageRenderModelRequest(
        source: lastRequest.source,
        documentId: descriptor.id,
        pageIndex: pageIndex,
        options: lastRequest.options,
      ),
    );

    switch (result) {
      case viewer_platform.GetPageRenderModelSuccess(page: final page):
        currentPage = page;
        currentPageIndex = page.pageIndex;
        pageError = null;
        pageStatus = ViewerPageStatus.ready;
      case viewer_platform.GetPageRenderModelFailure(
        error: final pageFetchError,
      ):
        currentPage = null;
        pageError = pageFetchError.toViewerException();
        pageStatus = ViewerPageStatus.error;
    }

    notifyListeners();
  }

  Future<void> submitPassword(String password) async {
    final lastRequest = _lastRequest;
    if (lastRequest == null) {
      return;
    }

    await openDocument(
      viewer_platform.OpenDocumentRequest(
        source: lastRequest.source,
        options: viewer_platform.OpenOptions(
          password: password,
          preferLazyLoading: lastRequest.options.preferLazyLoading,
        ),
      ),
    );
  }
}
