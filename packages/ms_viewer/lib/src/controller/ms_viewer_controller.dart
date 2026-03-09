import 'dart:math' as math;
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
  static const int _xlsxTileRows = 48;
  static const int _xlsxTileColumns = 16;
  static const int _maxCachedSheetTiles = 24;
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
  final Map<String, viewer_platform.PageRenderModel> _sheetTileCache = {};
  final List<String> _sheetTileUsageOrder = <String>[];
  final Set<int> _loadingPageIndexes = <int>{};
  viewer_platform.SheetWindow _currentSheetWindow = _defaultSheetWindow;
  bool _sheetWindowLoading = false;
  List<Rect> pageHighlights = const [];
  SearchSheetCell? _activeSheetCell;
  int? _activeSheetCellPageIndex;
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
  SearchSheetCell? get activeSheetCell =>
      _activeSheetCellPageIndex == currentPageIndex ? _activeSheetCell : null;
  String? get activeSheetCellLabel {
    final cell = activeSheetCell;
    if (cell == null) {
      return null;
    }
    return '${_columnLabel(cell.column)}${cell.row}';
  }

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
    _sheetTileCache.clear();
    _sheetTileUsageOrder.clear();
    _loadingPageIndexes.clear();
    pageError = null;
    pageHighlights = const [];
    _activeSheetCell = null;
    _activeSheetCellPageIndex = null;
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
    _sheetTileCache.clear();
    _sheetTileUsageOrder.clear();
    _loadingPageIndexes.clear();
    pageError = null;
    pageHighlights = const [];
    _activeSheetCell = null;
    _activeSheetCellPageIndex = null;
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
        _sheetTileCache.clear();
        _sheetTileUsageOrder.clear();
        _loadingPageIndexes.clear();
        pageError = null;
        pageHighlights = const [];
        _activeSheetCell = null;
        _activeSheetCellPageIndex = null;
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
    _applySearchHighlightOnly();
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
    final normalizedWindow = _normalizeXlsxWindowToTileUnion(window);
    if (_sameSheetWindowBounds(previousWindow, normalizedWindow)) {
      return;
    }

    final cacheKey = _sheetWindowCacheKey(pageIndex, normalizedWindow);
    final cachedPage = _sheetWindowCache[cacheKey];
    _currentSheetWindow = normalizedWindow;
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

    final page = await _fetchXlsxWindowPage(
      source: lastRequest.source,
      documentId: descriptor.id,
      pageIndex: pageIndex,
      options: lastRequest.options,
      requestedWindow: normalizedWindow,
    );

    if (page != null) {
      _cacheXlsxWindowPage(page);
      currentPage = page;
      currentPageIndex = page.pageIndex;
      final viewport = page.sheetViewport;
      if (viewport != null) {
        _currentSheetWindow = viewer_platform.SheetWindow(
          startRow: viewport.window.startRow,
          endRow: viewport.window.endRow,
          startColumn: viewport.window.startColumn,
          endColumn: viewport.window.endColumn,
        );
      }
      pageError = null;
      _applySearchHighlightOnly();
      pageStatus = ViewerPageStatus.ready;
    } else {
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
    viewer_platform.SheetWindow? sheetWindowOverride,
  }) {
    return viewer_platform.GetPageRenderModelRequest(
      source: source,
      documentId: documentId,
      pageIndex: pageIndex,
      sheetWindow: kind == DocumentKind.xlsx
          ? (sheetWindowOverride ?? _currentSheetWindow)
          : null,
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
                  sheetCell: match.sheetCell == null
                      ? null
                      : SearchSheetCell(
                          row: match.sheetCell!.row,
                          column: match.sheetCell!.column,
                        ),
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

  void selectSheetCellAt(Offset pagePosition) {
    final page = currentPage;
    final pageIndex = currentPageIndex;
    if (page == null || pageIndex == null || document?.kind != DocumentKind.xlsx) {
      return;
    }

    final selectedCell = page.sheetCells.cast<viewer_platform.SheetCellModel?>()
        .firstWhere(
          (cell) =>
              cell != null &&
              pagePosition.dx >= cell.bounds.x &&
              pagePosition.dx <= cell.bounds.x + cell.bounds.width &&
              pagePosition.dy >= cell.bounds.y &&
              pagePosition.dy <= cell.bounds.y + cell.bounds.height,
          orElse: () => null,
        );
    if (selectedCell == null) {
      return;
    }

    _activeSheetCell = SearchSheetCell(
      row: selectedCell.row,
      column: selectedCell.column,
    );
    _activeSheetCellPageIndex = pageIndex;
    selectionController.clear();
    _applySearchHighlightOnly();
    notifyListeners();
  }

  Future<void> moveActiveSheetCell({
    required int rowDelta,
    required int columnDelta,
  }) async {
    if (document?.kind != DocumentKind.xlsx) {
      return;
    }
    final page = currentPage;
    final pageIndex = currentPageIndex;
    if (page == null || pageIndex == null) {
      return;
    }

    final anchorCell =
        activeSheetCell ??
        _firstVisibleSheetCell(page) ??
        const SearchSheetCell(row: 1, column: 1);
    const maxRow = 1_048_576;
    const maxColumn = 16_384;
    final nextCell = SearchSheetCell(
      row: (anchorCell.row + rowDelta).clamp(1, maxRow),
      column: (anchorCell.column + columnDelta).clamp(1, maxColumn),
    );

    _activeSheetCell = nextCell;
    _activeSheetCellPageIndex = pageIndex;

    if (!_sheetCellInWindow(_currentSheetWindow, nextCell)) {
      await loadSheetWindow(_buildWindowAroundSheetCell(nextCell));
      return;
    }

    _applySearchHighlightOnly();
    notifyListeners();
  }

  Future<void> _syncSearchHighlights() async {
    final result = searchController.currentResult;
    if (result == null) {
      _applySearchHighlightOnly();
      return;
    }

    final descriptor = document;
    if (descriptor?.kind == DocumentKind.xlsx && result.sheetCell != null) {
      _activeSheetCell = result.sheetCell;
      _activeSheetCellPageIndex = result.pageIndex;
      if (currentPageIndex != result.pageIndex) {
        await loadPage(result.pageIndex);
      }
      final sheetCell = result.sheetCell!;
      if (!_sheetCellInWindow(_currentSheetWindow, sheetCell)) {
        await loadSheetWindow(_buildWindowAroundSheetCell(sheetCell));
      }
    } else if (currentPageIndex != result.pageIndex) {
      await loadSelectionPage(result.pageIndex);
    }
    _applySearchHighlightOnly();
  }

  void _applySearchHighlightOnly() {
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

    if (descriptor.kind == DocumentKind.xlsx) {
      final page = await _fetchXlsxWindowPage(
        source: lastRequest.source,
        documentId: descriptor.id,
        pageIndex: pageIndex,
        options: lastRequest.options,
        requestedWindow: _normalizeXlsxWindowToTileUnion(_currentSheetWindow),
      );
      _loadingPageIndexes.remove(pageIndex);

      if (page != null) {
        _pageCache[page.pageIndex] = page;
        _cacheXlsxWindowPage(page);
        if (!updateVisibleState) {
          notifyListeners();
        }
        return page;
      }

      if (updateVisibleState) {
        currentPage = null;
        pageError = const viewer_platform.OpenDocumentError(
          code: viewer_platform.ViewerErrorCode.invalidDocument,
          message: 'Failed to load sheet preview.',
        ).toViewerException();
        pageHighlights = const [];
        pageStatus = ViewerPageStatus.error;
        notifyListeners();
      }
      return null;
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
    final page = currentPage;
    if (page == null) {
      pageHighlights = const [];
      return;
    }

    final highlights = <Rect>[];

    final activeCell = activeSheetCell;
    if (activeCell != null) {
      for (final cell in page.sheetCells) {
        if (cell.row == activeCell.row && cell.column == activeCell.column) {
          highlights.add(
            Rect.fromLTWH(
              cell.bounds.x,
              cell.bounds.y,
              cell.bounds.width,
              cell.bounds.height,
            ),
          );
        }
      }
    }

    final result = searchController.currentResult;
    if (result != null && currentPageIndex == result.pageIndex) {
      if (document?.kind == DocumentKind.xlsx && result.sheetCell != null) {
        for (final cell in page.sheetCells) {
          if (cell.row == result.sheetCell!.row &&
              cell.column == result.sheetCell!.column) {
            final bounds = Rect.fromLTWH(
              cell.bounds.x,
              cell.bounds.y,
              cell.bounds.width,
              cell.bounds.height,
            );
            if (!highlights.contains(bounds)) {
              highlights.add(bounds);
            }
          }
        }
      } else {
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
            final bounds = Rect.fromLTWH(
              node.bounds.x,
              node.bounds.y,
              node.bounds.width,
              node.bounds.height,
            );
            if (!highlights.contains(bounds)) {
              highlights.add(bounds);
            }
          }
        }
      }
    }

    final selectionRect = selectionController.selectionRect;
    if (selectionRect != null) {
      for (final node in page.nodes) {
        if (node is! viewer_platform.TextRenderNodeModel) {
          continue;
        }
        final bounds = Rect.fromLTWH(
          node.bounds.x,
          node.bounds.y,
          node.bounds.width,
          node.bounds.height,
        );
        if (selectionRect.overlaps(bounds) && !highlights.contains(bounds)) {
          highlights.add(bounds);
        }
      }
    }

    pageHighlights = highlights;
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

  Future<viewer_platform.PageRenderModel?> _fetchSinglePage({
    required viewer_platform.OpenDocumentSource source,
    required String documentId,
    required int pageIndex,
    required viewer_platform.OpenOptions options,
    required DocumentKind kind,
    viewer_platform.SheetWindow? sheetWindowOverride,
  }) async {
    final result = await _platform.getPageRenderModel(
      _buildPageRequest(
        source: source,
        documentId: documentId,
        pageIndex: pageIndex,
        options: options,
        kind: kind,
        sheetWindowOverride: sheetWindowOverride,
      ),
    );

    switch (result) {
      case viewer_platform.GetPageRenderModelSuccess(page: final page):
        return page;
      case viewer_platform.GetPageRenderModelFailure():
        return null;
    }
  }

  Future<viewer_platform.PageRenderModel?> _fetchXlsxWindowPage({
    required viewer_platform.OpenDocumentSource source,
    required String documentId,
    required int pageIndex,
    required viewer_platform.OpenOptions options,
    required viewer_platform.SheetWindow requestedWindow,
  }) async {
    final normalizedWindow = _normalizeXlsxWindowToTileUnion(requestedWindow);
    final cacheKey = _sheetWindowCacheKey(pageIndex, normalizedWindow);
    final cachedWindowPage = _sheetWindowCache[cacheKey];
    if (cachedWindowPage != null) {
      return cachedWindowPage;
    }

    final tileWindows = _resolveXlsxTileWindows(normalizedWindow);
    final tilePages = <viewer_platform.PageRenderModel>[];
    for (final tileWindow in tileWindows) {
      final tileKey = _sheetTileCacheKey(pageIndex, tileWindow);
      final cachedTile = _sheetTileCache[tileKey];
      if (cachedTile != null) {
        _touchSheetTileKey(tileKey);
        tilePages.add(cachedTile);
        continue;
      }

      final page = await _fetchSinglePage(
        source: source,
        documentId: documentId,
        pageIndex: pageIndex,
        options: options,
        kind: DocumentKind.xlsx,
        sheetWindowOverride: tileWindow,
      );
      if (page == null) {
        return null;
      }
      _cacheSheetTilePage(page);
      tilePages.add(page);
    }

    final compositePage = tilePages.length == 1
        ? tilePages.single
        : _composeXlsxTilePages(pageIndex, tilePages);
    _sheetWindowCache[cacheKey] = compositePage;
    return compositePage;
  }

  viewer_platform.PageRenderModel _composeXlsxTilePages(
    int pageIndex,
    List<viewer_platform.PageRenderModel> tilePages,
  ) {
    final pages = [...tilePages]
      ..sort((a, b) {
        final aWindow = a.sheetViewport!.window;
        final bWindow = b.sheetViewport!.window;
        final rowCompare = aWindow.startRow.compareTo(bWindow.startRow);
        if (rowCompare != 0) {
          return rowCompare;
        }
        return aWindow.startColumn.compareTo(bWindow.startColumn);
      });

    final rowBandHeights = <int, double>{};
    final columnBandWidths = <int, double>{};
    final visibleRows = <int>{};
    final visibleColumns = <int>{};

    for (final page in pages) {
      final viewport = page.sheetViewport!;
      rowBandHeights.putIfAbsent(viewport.window.startRow, () => page.height);
      columnBandWidths.putIfAbsent(viewport.window.startColumn, () => page.width);
      visibleRows.addAll(viewport.visibleRows);
      visibleColumns.addAll(viewport.visibleColumns);
    }

    final sortedRowStarts = rowBandHeights.keys.toList()..sort();
    final sortedColumnStarts = columnBandWidths.keys.toList()..sort();
    final rowOffsets = <int, double>{};
    final columnOffsets = <int, double>{};

    var currentYOffset = 0.0;
    for (final rowStart in sortedRowStarts) {
      rowOffsets[rowStart] = currentYOffset;
      currentYOffset += rowBandHeights[rowStart]!;
    }

    var currentXOffset = 0.0;
    for (final columnStart in sortedColumnStarts) {
      columnOffsets[columnStart] = currentXOffset;
      currentXOffset += columnBandWidths[columnStart]!;
    }

    final nodes = <viewer_platform.RenderNodeModel>[];
    final selectionAnchors = <viewer_platform.SelectionAnchorModel>[];
    final sheetCells = <viewer_platform.SheetCellModel>[];

    for (final page in pages) {
      final viewport = page.sheetViewport!;
      final dx = columnOffsets[viewport.window.startColumn] ?? 0.0;
      final dy = rowOffsets[viewport.window.startRow] ?? 0.0;
      final nodeIndexOffset = nodes.length;

      nodes.addAll(
        page.nodes.map((node) => _offsetRenderNode(node, dx: dx, dy: dy)),
      );
      selectionAnchors.addAll(
        page.selectionAnchors.map(
          (anchor) => viewer_platform.SelectionAnchorModel(
            nodeIndex: anchor.nodeIndex + nodeIndexOffset,
            charIndex: anchor.charIndex,
            x: anchor.x + dx,
            y: anchor.y + dy,
          ),
        ),
      );
      sheetCells.addAll(
        page.sheetCells.map(
          (cell) => viewer_platform.SheetCellModel(
            row: cell.row,
            column: cell.column,
            bounds: _offsetRect(cell.bounds, dx: dx, dy: dy),
          ),
        ),
      );
    }

    final firstViewport = pages.first.sheetViewport!;
    final lastViewport = pages.last.sheetViewport!;
    final sortedVisibleRows = visibleRows.toList()..sort();
    final sortedVisibleColumns = visibleColumns.toList()..sort();

    return viewer_platform.PageRenderModel(
      pageIndex: pageIndex,
      width: currentXOffset,
      height: currentYOffset,
      nodes: nodes,
      selectionAnchors: selectionAnchors,
      sheetViewport: viewer_platform.SheetViewportModel(
        window: viewer_platform.SheetBoundsModel(
          startRow: firstViewport.window.startRow,
          endRow: lastViewport.window.endRow,
          startColumn: firstViewport.window.startColumn,
          endColumn: pages
              .map((page) => page.sheetViewport!.window.endColumn)
              .reduce(math.max),
        ),
        effectiveBounds: firstViewport.effectiveBounds,
        frozenPane: firstViewport.frozenPane,
        visibleRows: sortedVisibleRows,
        visibleColumns: sortedVisibleColumns,
      ),
      sheetCells: sheetCells,
    );
  }

  viewer_platform.RenderNodeModel _offsetRenderNode(
    viewer_platform.RenderNodeModel node, {
    required double dx,
    required double dy,
  }) {
    return switch (node) {
      viewer_platform.TextRenderNodeModel() => viewer_platform.TextRenderNodeModel(
        text: node.text,
        bounds: _offsetRect(node.bounds, dx: dx, dy: dy),
        style: node.style,
        range: node.range,
      ),
      viewer_platform.ImageRenderNodeModel() =>
        viewer_platform.ImageRenderNodeModel(
          resourceId: node.resourceId,
          description: node.description,
          contentType: node.contentType,
          dataBase64: node.dataBase64,
          bounds: _offsetRect(node.bounds, dx: dx, dy: dy),
          crop: node.crop,
          flipHorizontal: node.flipHorizontal,
          flipVertical: node.flipVertical,
        ),
      viewer_platform.BoxRenderNodeModel() => viewer_platform.BoxRenderNodeModel(
        bounds: _offsetRect(node.bounds, dx: dx, dy: dy),
        fillColorHex: node.fillColorHex,
        gradientEndColorHex: node.gradientEndColorHex,
        gradientAngleDegrees: node.gradientAngleDegrees,
        strokeColorHex: node.strokeColorHex,
        strokeWidth: node.strokeWidth,
        cornerRadius: node.cornerRadius,
      ),
    };
  }

  viewer_platform.RectModel _offsetRect(
    viewer_platform.RectModel rect, {
    required double dx,
    required double dy,
  }) {
    return viewer_platform.RectModel(
      x: rect.x + dx,
      y: rect.y + dy,
      width: rect.width,
      height: rect.height,
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

  void _cacheSheetTilePage(viewer_platform.PageRenderModel page) {
    final viewport = page.sheetViewport;
    if (viewport == null) {
      return;
    }
    final tileWindow = viewer_platform.SheetWindow(
      startRow: viewport.window.startRow,
      endRow: viewport.window.endRow,
      startColumn: viewport.window.startColumn,
      endColumn: viewport.window.endColumn,
    );
    final tileKey = _sheetTileCacheKey(page.pageIndex, tileWindow);
    _sheetTileCache[tileKey] = page;
    _touchSheetTileKey(tileKey);
    while (_sheetTileUsageOrder.length > _maxCachedSheetTiles) {
      final evictedKey = _sheetTileUsageOrder.removeAt(0);
      _sheetTileCache.remove(evictedKey);
    }
  }

  void _touchSheetTileKey(String key) {
    _sheetTileUsageOrder.remove(key);
    _sheetTileUsageOrder.add(key);
  }

  String _sheetWindowCacheKey(
    int pageIndex,
    viewer_platform.SheetWindow window,
  ) {
    return '$pageIndex:${window.startRow}:${window.endRow}:${window.startColumn}:${window.endColumn}';
  }

  String _sheetTileCacheKey(int pageIndex, viewer_platform.SheetWindow window) {
    return 'tile:${_sheetWindowCacheKey(pageIndex, window)}';
  }

  viewer_platform.SheetWindow _normalizeXlsxWindowToTileUnion(
    viewer_platform.SheetWindow window,
  ) {
    final tiles = _resolveXlsxTileWindows(window);
    final firstTile = tiles.first;
    final lastTile = tiles.last;
    return viewer_platform.SheetWindow(
      startRow: firstTile.startRow,
      endRow: tiles.map((tile) => tile.endRow).reduce(math.max),
      startColumn: firstTile.startColumn,
      endColumn: tiles.map((tile) => tile.endColumn).reduce(math.max),
    );
  }

  List<viewer_platform.SheetWindow> _resolveXlsxTileWindows(
    viewer_platform.SheetWindow window,
  ) {
    const maxRows = 1_048_576;
    const maxColumns = 16_384;
    final normalizedStartRow =
        (((window.startRow - 1) ~/ _xlsxTileRows) * _xlsxTileRows) + 1;
    final normalizedStartColumn =
        (((window.startColumn - 1) ~/ _xlsxTileColumns) * _xlsxTileColumns) + 1;
    final normalizedEndRow =
        (((window.endRow - 1) ~/ _xlsxTileRows) * _xlsxTileRows) + 1;
    final normalizedEndColumn =
        (((window.endColumn - 1) ~/ _xlsxTileColumns) * _xlsxTileColumns) + 1;

    final tiles = <viewer_platform.SheetWindow>[];
    for (
      var rowStart = normalizedStartRow;
      rowStart <= normalizedEndRow;
      rowStart += _xlsxTileRows
    ) {
      for (
        var columnStart = normalizedStartColumn;
        columnStart <= normalizedEndColumn;
        columnStart += _xlsxTileColumns
      ) {
        tiles.add(
          viewer_platform.SheetWindow(
            startRow: rowStart,
            endRow: math.min(rowStart + _xlsxTileRows - 1, maxRows),
            startColumn: columnStart,
            endColumn: math.min(
              columnStart + _xlsxTileColumns - 1,
              maxColumns,
            ),
          ),
        );
      }
    }
    return tiles;
  }

  bool _sheetCellInWindow(
    viewer_platform.SheetWindow window,
    SearchSheetCell cell,
  ) {
    return cell.row >= window.startRow &&
        cell.row <= window.endRow &&
        cell.column >= window.startColumn &&
        cell.column <= window.endColumn;
  }

  viewer_platform.SheetWindow _buildWindowAroundSheetCell(SearchSheetCell cell) {
    const maxRows = 1_048_576;
    const maxColumns = 16_384;
    final rowCount = _currentSheetWindow.endRow - _currentSheetWindow.startRow + 1;
    final columnCount =
        _currentSheetWindow.endColumn - _currentSheetWindow.startColumn + 1;
    final halfRows = rowCount ~/ 2;
    final halfColumns = columnCount ~/ 2;
    final startRow = (cell.row - halfRows).clamp(1, maxRows);
    final startColumn = (cell.column - halfColumns).clamp(1, maxColumns);
    final endRow = (startRow + rowCount - 1).clamp(1, maxRows);
    final endColumn = (startColumn + columnCount - 1).clamp(1, maxColumns);
    return viewer_platform.SheetWindow(
      startRow: startRow,
      endRow: endRow,
      startColumn: startColumn,
      endColumn: endColumn,
    );
  }

  SearchSheetCell? _firstVisibleSheetCell(viewer_platform.PageRenderModel page) {
    if (page.sheetCells.isEmpty) {
      return null;
    }
    final cells = [...page.sheetCells]
      ..sort((a, b) {
        final rowCompare = a.row.compareTo(b.row);
        if (rowCompare != 0) {
          return rowCompare;
        }
        return a.column.compareTo(b.column);
      });
    final first = cells.first;
    return SearchSheetCell(row: first.row, column: first.column);
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

String _columnLabel(int index) {
  var value = index;
  final buffer = StringBuffer();
  while (value > 0) {
    final remainder = (value - 1) % 26;
    buffer.writeCharCode(65 + remainder);
    value = (value - 1) ~/ 26;
  }
  return buffer.toString().split('').reversed.join();
}
