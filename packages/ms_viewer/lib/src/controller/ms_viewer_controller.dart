import 'dart:ui';

import 'package:flutter/foundation.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart'
    as viewer_platform;

import '../errors/open_document_error_mapper.dart';
import '../models/document_descriptor.dart';
import '../models/search_result.dart';
import '../password/password_prompt_state.dart';
import '../search/document_search_controller.dart';
import '../selection/selection_drag_controller.dart';

enum ViewerShellStatus { idle, loading, ready, passwordPrompt, error }

enum ViewerPageStatus { idle, loading, ready, error }

class MsViewerController extends ChangeNotifier {
  static const viewer_platform.SheetWindow _defaultSheetWindow =
      viewer_platform.SheetWindow(
        startRow: 1,
        endRow: 40,
        startColumn: 1,
        endColumn: 16,
      );

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
  final Map<int, viewer_platform.PageRenderModel> _pageCache = {};
  final Map<String, viewer_platform.PageRenderModel> _sheetWindowCache = {};
  final Set<int> _loadingPageIndexes = <int>{};
  viewer_platform.SheetWindow _currentSheetWindow = _defaultSheetWindow;
  bool _sheetWindowLoading = false;
  List<Rect> pageHighlights = const [];
  final DocumentSearchController searchController = DocumentSearchController();
  final SelectionDragController selectionController = SelectionDragController();
  PasswordPromptState passwordPromptState = const PasswordPromptState(
    status: PasswordPromptStatus.idle,
  );
  ViewerShellStatus status = ViewerShellStatus.idle;
  ViewerPageStatus pageStatus = ViewerPageStatus.idle;
  viewer_platform.OpenDocumentRequest? _lastRequest;

  int get pageCount => document?.pageCount ?? 0;

  bool get canGoToPreviousPage =>
      pageCount > 0 &&
      (currentPageIndex ?? 0) > 0 &&
      pageStatus != ViewerPageStatus.loading;

  bool get canGoToNextPage =>
      pageCount > 0 &&
      (currentPageIndex ?? 0) < pageCount - 1 &&
      pageStatus != ViewerPageStatus.loading;

  viewer_platform.SheetWindow get currentSheetWindow => _currentSheetWindow;
  bool get isSheetWindowLoading => _sheetWindowLoading;

  viewer_platform.PageRenderModel? pageForIndex(int pageIndex) {
    return _pageCache[pageIndex];
  }

  bool isPageLoading(int pageIndex) => _loadingPageIndexes.contains(pageIndex);

  void attachDocument(DocumentDescriptor next) {
    document = next;
    error = null;
    currentPage = null;
    currentPageIndex = null;
    _pageCache.clear();
    _sheetWindowCache.clear();
    _loadingPageIndexes.clear();
    pageError = null;
    pageHighlights = const [];
    pageStatus = ViewerPageStatus.idle;
    _currentSheetWindow = _defaultSheetWindow;
    _sheetWindowLoading = false;
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
    _pageCache.clear();
    _sheetWindowCache.clear();
    _loadingPageIndexes.clear();
    pageError = null;
    pageHighlights = const [];
    pageStatus = ViewerPageStatus.idle;
    _currentSheetWindow = _defaultSheetWindow;
    _sheetWindowLoading = false;
    searchController.clear();
    selectionController.clear();
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
        if (_supportsImmediatePageFetch(opened.kind) && opened.pageCount > 0) {
          final initialPageIndex = document?.activePageIndex ?? 0;
          await loadPage(initialPageIndex);
        }
      case viewer_platform.OpenDocumentFailure(error: final openError):
        document = null;
        currentPage = null;
        currentPageIndex = null;
        _pageCache.clear();
        _sheetWindowCache.clear();
        _loadingPageIndexes.clear();
        pageError = null;
        pageHighlights = const [];
        pageStatus = ViewerPageStatus.idle;
        searchController.clear();
        selectionController.clear();
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
    final page = await _fetchPage(pageIndex, updateVisibleState: true);
    if (page == null) {
      return;
    }

    currentPage = page;
    currentPageIndex = page.pageIndex;
    final descriptor = document;
    final sheetViewport = page.sheetViewport;
    if (descriptor?.kind == DocumentKind.xlsx && sheetViewport != null) {
      _currentSheetWindow = viewer_platform.SheetWindow(
        startRow: sheetViewport.window.startRow,
        endRow: sheetViewport.window.endRow,
        startColumn: sheetViewport.window.startColumn,
        endColumn: sheetViewport.window.endColumn,
      );
    }
    pageError = null;
    pageHighlights = const [];
    pageStatus = ViewerPageStatus.ready;
    _sheetWindowLoading = false;
    notifyListeners();
  }

  Future<void> ensurePageLoaded(int pageIndex) async {
    await _fetchPage(pageIndex, updateVisibleState: false);
  }

  void activateCachedPage(int pageIndex) {
    final lastRequest = _lastRequest;
    final descriptor = document;
    if (lastRequest == null || descriptor == null) {
      return;
    }
    final cached = _pageCache[pageIndex];
    if (cached == null) {
      return;
    }
    if (currentPageIndex == pageIndex && currentPage == cached) {
      return;
    }
    currentPage = cached;
    currentPageIndex = pageIndex;
    _applySearchHighlightOnly();
    notifyListeners();
  }

  Future<void> loadSheetWindow(viewer_platform.SheetWindow window) async {
    if (document?.kind != DocumentKind.xlsx) {
      return;
    }
    final descriptor = document;
    final lastRequest = _lastRequest;
    if (descriptor == null || lastRequest == null) {
      return;
    }

    final pageIndex = currentPageIndex ?? descriptor.activePageIndex ?? 0;
    final previousWindow = _currentSheetWindow;
    if (_sameSheetWindowBounds(previousWindow, window)) {
      return;
    }

    final cacheKey = _sheetWindowCacheKey(pageIndex, window);
    final cachedPage = _sheetWindowCache[cacheKey];
    _currentSheetWindow = window;
    if (cachedPage != null) {
      currentPage = cachedPage;
      currentPageIndex = cachedPage.pageIndex;
      _pageCache[cachedPage.pageIndex] = cachedPage;
      pageError = null;
      pageStatus = ViewerPageStatus.ready;
      _sheetWindowLoading = false;
      notifyListeners();
      return;
    }

    _sheetWindowLoading = true;
    notifyListeners();

    final result = await _platform.getPageRenderModel(
      _buildPageRequest(
        source: lastRequest.source,
        documentId: descriptor.id,
        pageIndex: pageIndex,
        options: lastRequest.options,
        kind: descriptor.kind,
      ),
    );

    switch (result) {
      case viewer_platform.GetPageRenderModelSuccess(page: final page):
        _cacheXlsxWindowPage(page);
        currentPage = page;
        currentPageIndex = page.pageIndex;
        pageError = null;
        pageStatus = ViewerPageStatus.ready;
      case viewer_platform.GetPageRenderModelFailure():
        _currentSheetWindow = previousWindow;
        if (currentPage == null) {
          pageStatus = ViewerPageStatus.error;
        }
    }

    _sheetWindowLoading = false;
    notifyListeners();
  }

  Future<void> goToPreviousPage() async {
    final index = currentPageIndex ?? 0;
    if (index <= 0) {
      return;
    }
    await loadPage(index - 1);
  }

  Future<void> goToNextPage() async {
    final descriptor = document;
    final index = currentPageIndex ?? 0;
    if (descriptor == null || index >= descriptor.pageCount - 1) {
      return;
    }
    await loadPage(index + 1);
  }

  bool _supportsImmediatePageFetch(viewer_platform.DocumentKind kind) {
    return kind == viewer_platform.DocumentKind.docx ||
        kind == viewer_platform.DocumentKind.pptx ||
        kind == viewer_platform.DocumentKind.xlsx;
  }

  viewer_platform.GetPageRenderModelRequest _buildPageRequest({
    required viewer_platform.OpenDocumentSource source,
    required String documentId,
    required int pageIndex,
    required viewer_platform.OpenOptions options,
    required DocumentKind kind,
  }) {
    return viewer_platform.GetPageRenderModelRequest(
      source: source,
      documentId: documentId,
      pageIndex: pageIndex,
      sheetWindow: kind == DocumentKind.xlsx ? _currentSheetWindow : null,
      options: options,
    );
  }

  Future<void> search(String query) async {
    final lastRequest = _lastRequest;
    final descriptor = document;
    if (lastRequest == null || descriptor == null) {
      return;
    }

    final result = await _platform.searchDocument(
      viewer_platform.SearchDocumentRequest(
        source: lastRequest.source,
        documentId: descriptor.id,
        query: query,
        options: lastRequest.options,
      ),
    );

    switch (result) {
      case viewer_platform.SearchDocumentSuccess(matches: final matches):
        searchController.updateResults(
          query: query,
          results: matches
              .map(
                (match) => SearchResult(
                  query: match.query,
                  pageIndex: match.pageIndex,
                  start: match.start,
                  end: match.end,
                  preview: match.preview,
                ),
              )
              .toList(growable: false),
        );
        await _syncSearchHighlights();
      case viewer_platform.SearchDocumentFailure(error: final searchError):
        searchController.clear();
        pageHighlights = const [];
        pageError = searchError.toViewerException();
    }

    notifyListeners();
  }

  Future<void> selectSearchResult(int index) async {
    searchController.select(index);
    await _syncSearchHighlights();
    notifyListeners();
  }

  Future<void> loadSelectionPage(int pageIndex) async {
    final lastRequest = _lastRequest;
    final descriptor = document;
    if (lastRequest == null || descriptor == null) {
      return;
    }

    pageStatus = ViewerPageStatus.loading;
    currentPageIndex = pageIndex;
    pageError = null;
    pageHighlights = const [];
    notifyListeners();

    final result = await _platform.getSelectionPage(
      viewer_platform.GetSelectionPageRequest(
        source: lastRequest.source,
        documentId: descriptor.id,
        pageIndex: pageIndex,
        options: lastRequest.options,
      ),
    );

    switch (result) {
      case viewer_platform.GetSelectionPageSuccess(page: final page):
        currentPage = page;
        currentPageIndex = page.pageIndex;
        _pageCache[page.pageIndex] = page;
        pageError = null;
        pageStatus = ViewerPageStatus.ready;
      case viewer_platform.GetSelectionPageFailure(error: final selectionError):
        currentPage = null;
        pageError = selectionError.toViewerException();
        pageStatus = ViewerPageStatus.error;
    }

    notifyListeners();
  }

  void startSelectionAt(Offset pagePosition) {
    selectionController.start(pagePosition);
    _applySelectionHighlights();
    notifyListeners();
  }

  void updateSelectionAt(Offset pagePosition) {
    selectionController.update(pagePosition);
    _applySelectionHighlights();
    notifyListeners();
  }

  void clearSelection() {
    selectionController.clear();
    _applySearchHighlightOnly();
    notifyListeners();
  }

  Future<void> _syncSearchHighlights() async {
    final result = searchController.currentResult;
    if (result == null) {
      _applySearchHighlightOnly();
      return;
    }

    if (currentPageIndex != result.pageIndex) {
      await loadSelectionPage(result.pageIndex);
    }
    _applySearchHighlightOnly();
  }

  void _applySearchHighlightOnly() {
    final page = currentPage;
    final result = searchController.currentResult;
    if (page == null ||
        result == null ||
        currentPageIndex != result.pageIndex) {
      pageHighlights = const [];
      return;
    }

    final highlights = <Rect>[];
    for (final node in page.nodes) {
      if (node is! viewer_platform.TextRenderNodeModel) {
        continue;
      }
      if (_rangesOverlap(
        node.range.start,
        node.range.end,
        result.start,
        result.end,
      )) {
        highlights.add(
          Rect.fromLTWH(
            node.bounds.x,
            node.bounds.y,
            node.bounds.width,
            node.bounds.height,
          ),
        );
      }
    }

    pageHighlights = highlights;
    _applySelectionHighlights();
  }

  Future<viewer_platform.PageRenderModel?> _fetchPage(
    int pageIndex, {
    required bool updateVisibleState,
  }) async {
    final lastRequest = _lastRequest;
    final descriptor = document;
    if (lastRequest == null || descriptor == null) {
      return null;
    }
    if (pageIndex < 0 || pageIndex >= descriptor.pageCount) {
      return null;
    }
    if (!updateVisibleState && _pageCache.containsKey(pageIndex)) {
      return _pageCache[pageIndex];
    }
    if (_loadingPageIndexes.contains(pageIndex)) {
      return _pageCache[pageIndex];
    }

    _loadingPageIndexes.add(pageIndex);
    if (updateVisibleState) {
      pageStatus = ViewerPageStatus.loading;
      currentPage = null;
      currentPageIndex = pageIndex;
      pageError = null;
      notifyListeners();
    }

    final result = await _platform.getPageRenderModel(
      _buildPageRequest(
        source: lastRequest.source,
        documentId: descriptor.id,
        pageIndex: pageIndex,
        options: lastRequest.options,
        kind: descriptor.kind,
      ),
    );
    _loadingPageIndexes.remove(pageIndex);

    switch (result) {
      case viewer_platform.GetPageRenderModelSuccess(page: final page):
        _pageCache[page.pageIndex] = page;
        if (descriptor.kind == DocumentKind.xlsx) {
          _cacheXlsxWindowPage(page);
        }
        if (!updateVisibleState) {
          notifyListeners();
        }
        return page;
      case viewer_platform.GetPageRenderModelFailure(
        error: final pageFetchError,
      ):
        if (updateVisibleState) {
          currentPage = null;
          pageError = pageFetchError.toViewerException();
          pageHighlights = const [];
          pageStatus = ViewerPageStatus.error;
          notifyListeners();
        }
        return null;
    }
  }

  void _applySelectionHighlights() {
    final selectionRect = selectionController.selectionRect;
    if (selectionRect == null || currentPage == null) {
      return;
    }

    final selectionHighlights = <Rect>[];
    for (final node in currentPage!.nodes) {
      if (node is! viewer_platform.TextRenderNodeModel) {
        continue;
      }
      final bounds = Rect.fromLTWH(
        node.bounds.x,
        node.bounds.y,
        node.bounds.width,
        node.bounds.height,
      );
      if (selectionRect.overlaps(bounds)) {
        selectionHighlights.add(bounds);
      }
    }

    final searchHighlights = pageHighlights
        .where((highlight) => !selectionHighlights.contains(highlight))
        .toList(growable: true);
    searchHighlights.addAll(selectionHighlights);
    pageHighlights = searchHighlights;
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

  void _cacheXlsxWindowPage(viewer_platform.PageRenderModel page) {
    final viewport = page.sheetViewport;
    if (viewport == null) {
      return;
    }
    _sheetWindowCache[
            _sheetWindowCacheKey(
              page.pageIndex,
              viewer_platform.SheetWindow(
                startRow: viewport.window.startRow,
                endRow: viewport.window.endRow,
                startColumn: viewport.window.startColumn,
                endColumn: viewport.window.endColumn,
              ),
            )] =
        page;
  }

  String _sheetWindowCacheKey(
    int pageIndex,
    viewer_platform.SheetWindow window,
  ) {
    return '$pageIndex:${window.startRow}:${window.endRow}:${window.startColumn}:${window.endColumn}';
  }
}

bool _rangesOverlap(int aStart, int aEnd, int bStart, int bEnd) {
  return aStart < bEnd && bStart < aEnd;
}

bool _sameSheetWindowBounds(
  viewer_platform.SheetWindow a,
  viewer_platform.SheetWindow b,
) {
  return a.startRow == b.startRow &&
      a.endRow == b.endRow &&
      a.startColumn == b.startColumn &&
      a.endColumn == b.endColumn;
}
