import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

import '../controller/ms_viewer_controller.dart';
import '../models/document_descriptor.dart' as document_model;
import '../models/search_result.dart';
import 'document_page_view.dart';
import 'search_result_list.dart';
import 'sheet_viewport.dart';

class MsDocumentView extends StatefulWidget {
  const MsDocumentView({
    super.key,
    required this.controller,
    this.previewPages = const [],
  });

  final MsViewerController controller;
  final List<PageRenderModel> previewPages;

  @override
  State<MsDocumentView> createState() => _MsDocumentViewState();
}

class _MsDocumentViewState extends State<MsDocumentView> {
  late final TextEditingController _passwordController;
  late final TextEditingController _searchController;
  late final FocusNode _sheetFocusNode;

  @override
  void initState() {
    super.initState();
    _passwordController = TextEditingController();
    _searchController = TextEditingController();
    _sheetFocusNode = FocusNode(debugLabel: 'xlsx-sheet-focus');
  }

  @override
  void dispose() {
    _passwordController.dispose();
    _searchController.dispose();
    _sheetFocusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: widget.controller,
      builder: (context, _) => _buildBody(context),
    );
  }

  Widget _buildBody(BuildContext context) {
    switch (widget.controller.status) {
      case ViewerShellStatus.loading:
        return const Center(child: CircularProgressIndicator());
      case ViewerShellStatus.passwordPrompt:
        return _buildPasswordPrompt(context);
      case ViewerShellStatus.error:
        return Center(
          child: Text(widget.controller.error?.message ?? 'Unknown error'),
        );
      case ViewerShellStatus.ready:
        final document = widget.controller.document;
        if (document == null) {
          return const Center(child: Text('No document attached'));
        }
        return _buildReadyState(document);
      case ViewerShellStatus.idle:
        final document = widget.controller.document;
        if (document == null) {
          return const Center(child: Text('No document attached'));
        }
        return _buildReadyState(document);
    }
  }

  Widget _buildReadyState(document_model.DocumentDescriptor document) {
    final title = document.title;
    final pageCount = document.pageCount;
    final isSpreadsheet = document.kind == document_model.DocumentKind.xlsx;
    final isContinuousPptx = document.kind == document_model.DocumentKind.pptx;
    final isContinuousDocx = document.kind == document_model.DocumentKind.docx;
    final isContinuousPagedDocument = isContinuousDocx || isContinuousPptx;
    final collectionLabel = isSpreadsheet
        ? 'sheets'
        : isContinuousPptx
        ? 'slides'
        : 'pages';
    final currentLabel = isSpreadsheet
        ? 'Sheet'
        : isContinuousPptx
        ? 'Slide'
        : 'Page';
    final fetchedPage = widget.controller.currentPage;
    final fallbackPreviewPage = widget.previewPages.isEmpty
        ? null
        : widget.previewPages.first;
    final page = fetchedPage ?? fallbackPreviewPage;
    final searchResults = widget.controller.searchController.results;
    if (isSpreadsheet) {
      return _buildSpreadsheetReadyState(
        document: document,
        title: title,
        pageCount: pageCount,
        page: page,
        searchResults: searchResults,
      );
    }

    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(title, style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 8),
          Text('$pageCount $collectionLabel'),
          const SizedBox(height: 16),
          if (!isContinuousPagedDocument) ...[
            Row(
              children: [
                OutlinedButton.icon(
                  onPressed: widget.controller.canGoToPreviousPage
                      ? () => unawaited(widget.controller.goToPreviousPage())
                      : null,
                  icon: const Icon(Icons.chevron_left),
                  label: const Text('Previous'),
                ),
                const SizedBox(width: 8),
                OutlinedButton.icon(
                  onPressed: widget.controller.canGoToNextPage
                      ? () => unawaited(widget.controller.goToNextPage())
                      : null,
                  icon: const Icon(Icons.chevron_right),
                  label: const Text('Next'),
                ),
                const SizedBox(width: 12),
                Text(
                  pageCount == 0
                      ? '$currentLabel 0 / 0'
                      : '$currentLabel ${(widget.controller.currentPageIndex ?? 0) + 1} / $pageCount',
                ),
              ],
            ),
            const SizedBox(height: 16),
          ],
          Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _searchController,
                  textInputAction: TextInputAction.search,
                  onSubmitted: _submitSearch,
                  decoration: const InputDecoration(
                    border: OutlineInputBorder(),
                    prefixIcon: Icon(Icons.search),
                    hintText: 'Search in document',
                  ),
                ),
              ),
              const SizedBox(width: 8),
              FilledButton(
                onPressed: () => _submitSearch(_searchController.text),
                child: const Text('Search'),
              ),
            ],
          ),
          if (_searchController.text.isNotEmpty) ...[
            const SizedBox(height: 12),
            SizedBox(
              height: 120,
              child: SearchResultList(
                results: searchResults,
                currentIndex: widget.controller.searchController.currentIndex,
                onTap: (index) {
                  unawaited(widget.controller.selectSearchResult(index));
                },
              ),
            ),
          ],
          const SizedBox(height: 16),
          Expanded(
            child: isContinuousPagedDocument
                ? _buildContinuousPagedSurface(
                    pageCount,
                    isPptx: isContinuousPptx,
                  )
                : switch (widget.controller.pageStatus) {
                    ViewerPageStatus.loading => const Center(
                      child: CircularProgressIndicator(),
                    ),
                    ViewerPageStatus.error => Center(
                      child: Text(
                        widget.controller.pageError?.message ??
                            'Failed to load page preview.',
                        textAlign: TextAlign.center,
                      ),
                    ),
                    ViewerPageStatus.ready when page != null =>
                      _buildViewerSurface(page, isSpreadsheet),
                    _ when page != null => _buildViewerSurface(
                      page,
                      isSpreadsheet,
                    ),
                    _ => const Center(
                      child: Text(
                        'Viewer placeholder: render model not loaded',
                        textAlign: TextAlign.center,
                      ),
                    ),
                  },
          ),
        ],
      ),
    );
  }

  Widget _buildContinuousPagedSurface(int pageCount, {required bool isPptx}) {
    if (pageCount == 0) {
      return Center(
        child: Text(isPptx ? 'No slides available.' : 'No pages available.'),
      );
    }
    if (widget.controller.pageStatus == ViewerPageStatus.error &&
        widget.controller.currentPage == null) {
      return Center(
        child: Text(
          widget.controller.pageError?.message ??
              'Failed to load page preview.',
          textAlign: TextAlign.center,
        ),
      );
    }

    return ListView.separated(
      key: ValueKey(isPptx ? 'pptx-slide-stack' : 'docx-page-stack'),
      padding: const EdgeInsets.only(bottom: 24),
      itemCount: pageCount,
      separatorBuilder: (_, __) => const SizedBox(height: 20),
      itemBuilder: (context, index) {
        final page = widget.controller.pageForIndex(index);
        if (page == null) {
          unawaited(widget.controller.ensurePageLoaded(index));
          return _buildPagedPlaceholder(index, isPptx: isPptx);
        }

        return Center(
          child: ConstrainedBox(
            constraints: BoxConstraints(maxWidth: isPptx ? 920 : 640),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Padding(
                  padding: const EdgeInsets.only(bottom: 8),
                  child: Text(
                    '${isPptx ? 'Slide' : 'Page'} ${index + 1}',
                    style: Theme.of(context).textTheme.bodySmall,
                    textAlign: TextAlign.right,
                  ),
                ),
                DocumentPageView(
                  page: page,
                  highlights: widget.controller.currentPageIndex == index
                      ? widget.controller.pageHighlights
                      : const [],
                  onSelectionStart: (pagePosition) {
                    widget.controller.activateCachedPage(index);
                    widget.controller.startSelectionAt(pagePosition);
                  },
                  onSelectionUpdate: (pagePosition) {
                    widget.controller.activateCachedPage(index);
                    widget.controller.updateSelectionAt(pagePosition);
                  },
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildPagedPlaceholder(int index, {required bool isPptx}) {
    return Center(
      child: ConstrainedBox(
        constraints: BoxConstraints(maxWidth: isPptx ? 920 : 640),
        child: AspectRatio(
          aspectRatio: isPptx ? 720 / 540 : 595 / 842,
          child: DecoratedBox(
            decoration: BoxDecoration(
              color: Colors.white,
              borderRadius: BorderRadius.circular(12),
              border: Border.all(color: const Color(0xFFD9E2EC)),
            ),
            child: Center(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const SizedBox(
                    width: 28,
                    height: 28,
                    child: CircularProgressIndicator(strokeWidth: 2.4),
                  ),
                  const SizedBox(height: 12),
                  Text('Loading ${isPptx ? 'slide' : 'page'} ${index + 1}'),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildSpreadsheetReadyState({
    required document_model.DocumentDescriptor document,
    required String title,
    required int pageCount,
    required PageRenderModel? page,
    required List<SearchResult> searchResults,
  }) {
    final sheetTabs = document.sheetTabs.isEmpty
        ? List<document_model.DocumentSheetTab>.generate(
            pageCount,
            (index) => document_model.DocumentSheetTab(
              pageIndex: index,
              title: 'Sheet ${index + 1}',
            ),
            growable: false,
          )
        : document.sheetTabs;
    final activeSheetIndex =
        widget.controller.currentPageIndex ??
        document.activePageIndex ??
        (sheetTabs.isEmpty ? 0 : sheetTabs.first.pageIndex);
    final activeSheetTitle = sheetTabs
        .cast<document_model.DocumentSheetTab?>()
        .firstWhere(
          (tab) => tab?.pageIndex == activeSheetIndex,
          orElse: () => sheetTabs.isEmpty ? null : sheetTabs.first,
        )
        ?.title;
    final activeCellLabel = widget.controller.activeSheetCellLabel;
    return Column(
      key: const ValueKey('xlsx-sheet-shell'),
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 16, 16, 12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(title, style: Theme.of(context).textTheme.titleMedium),
              const SizedBox(height: 6),
              Text(
                activeSheetTitle == null
                    ? '$pageCount sheets'
                    : '$activeSheetTitle · ${sheetTabs.length} sheets',
              ),
              if (activeCellLabel != null) ...[
                const SizedBox(height: 4),
                Text(
                  'Cell $activeCellLabel',
                  key: const ValueKey('xlsx-active-cell-label'),
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
              const SizedBox(height: 12),
              Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _searchController,
                      textInputAction: TextInputAction.search,
                      onSubmitted: _submitSearch,
                      decoration: const InputDecoration(
                        border: OutlineInputBorder(),
                        prefixIcon: Icon(Icons.search),
                        hintText: 'Search in document',
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),
                  FilledButton(
                    onPressed: () => _submitSearch(_searchController.text),
                    child: const Text('Search'),
                  ),
                ],
              ),
              if (_searchController.text.isNotEmpty) ...[
                const SizedBox(height: 12),
                SizedBox(
                  height: 120,
                  child: SearchResultList(
                    results: searchResults,
                    currentIndex:
                        widget.controller.searchController.currentIndex,
                    onTap: (index) {
                      unawaited(widget.controller.selectSearchResult(index));
                    },
                  ),
                ),
              ],
            ],
          ),
        ),
        Expanded(
          child: _buildSpreadsheetSurface(page),
        ),
        if (sheetTabs.isNotEmpty)
          Container(
            height: 56,
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: const BoxDecoration(
              color: Color(0xFFF8FBFF),
              border: Border(top: BorderSide(color: Color(0xFFD0D7DE))),
            ),
            child: ListView.separated(
              key: const ValueKey('xlsx-sheet-tabs'),
              scrollDirection: Axis.horizontal,
              itemCount: sheetTabs.length,
              separatorBuilder: (_, __) => const SizedBox(width: 8),
              itemBuilder: (context, index) {
                final tab = sheetTabs[index];
                final selected = tab.pageIndex == activeSheetIndex;
                return ChoiceChip(
                  key: ValueKey('xlsx-sheet-tab-${tab.pageIndex}'),
                  label: Text(tab.title),
                  selected: selected,
                  onSelected: (_) {
                    unawaited(widget.controller.loadPage(tab.pageIndex));
                  },
                );
              },
            ),
          ),
      ],
    );
  }

  Widget _buildViewerSurface(PageRenderModel page, bool isSpreadsheet) {
    final activeCellRect = isSpreadsheet
        ? _activeSpreadsheetCellRect(page)
        : null;
    final viewer = isSpreadsheet
        ? Focus(
            focusNode: _sheetFocusNode,
            autofocus: true,
            onKeyEvent: _handleSpreadsheetKeyEvent,
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: _sheetFocusNode.requestFocus,
              child: SheetViewport(
                page: page,
                highlights: widget.controller.pageHighlights,
                focusRect: activeCellRect,
                onCellTap: (pagePosition) {
                  _sheetFocusNode.requestFocus();
                  widget.controller.selectSheetCellAt(pagePosition);
                },
                onSelectionStart: widget.controller.startSelectionAt,
                onSelectionUpdate: widget.controller.updateSelectionAt,
                onWindowRequest: widget.controller.loadSheetWindow,
              ),
            ),
          )
        : DocumentPageView(
            page: page,
            highlights: widget.controller.pageHighlights,
            onSelectionStart: widget.controller.startSelectionAt,
            onSelectionUpdate: widget.controller.updateSelectionAt,
          );

    if (isSpreadsheet) {
      return ColoredBox(color: Colors.white, child: viewer);
    }

    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 640),
        child: viewer,
      ),
    );
  }

  Widget _buildSpreadsheetSurface(PageRenderModel? page) {
    if (page == null) {
      return switch (widget.controller.pageStatus) {
        ViewerPageStatus.loading => const Center(
          child: CircularProgressIndicator(),
        ),
        ViewerPageStatus.error => Center(
          child: Text(
            widget.controller.pageError?.message ??
                'Failed to load sheet preview.',
            textAlign: TextAlign.center,
          ),
        ),
        _ => const Center(
          child: Text(
            'Viewer placeholder: render model not loaded',
            textAlign: TextAlign.center,
          ),
        ),
      };
    }

    return Stack(
      children: [
        _buildViewerSurface(page, true),
        if (widget.controller.isSheetWindowLoading)
          Positioned(
            key: const ValueKey('xlsx-window-loading-indicator'),
            top: 12,
            right: 12,
            child: DecoratedBox(
              decoration: BoxDecoration(
                color: Colors.white.withOpacity(0.94),
                borderRadius: BorderRadius.circular(999),
                boxShadow: const [
                  BoxShadow(
                    color: Color(0x14000000),
                    blurRadius: 12,
                    offset: Offset(0, 6),
                  ),
                ],
              ),
              child: const Padding(
                padding: EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    SizedBox(
                      width: 14,
                      height: 14,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    ),
                    SizedBox(width: 8),
                    Text('Updating cells'),
                  ],
                ),
              ),
            ),
          ),
      ],
    );
  }

  Rect? _activeSpreadsheetCellRect(PageRenderModel page) {
    final activeCell = widget.controller.activeSheetCell;
    if (activeCell == null) {
      return null;
    }
    for (final cell in page.sheetCells) {
      if (cell.row == activeCell.row && cell.column == activeCell.column) {
        return Rect.fromLTWH(
          cell.bounds.x,
          cell.bounds.y,
          cell.bounds.width,
          cell.bounds.height,
        );
      }
    }
    return null;
  }

  KeyEventResult _handleSpreadsheetKeyEvent(
    FocusNode node,
    KeyEvent event,
  ) {
    if (event is! KeyDownEvent) {
      return KeyEventResult.ignored;
    }

    if (event.logicalKey == LogicalKeyboardKey.arrowLeft) {
      unawaited(
        widget.controller.moveActiveSheetCell(rowDelta: 0, columnDelta: -1),
      );
      return KeyEventResult.handled;
    }
    if (event.logicalKey == LogicalKeyboardKey.arrowRight) {
      unawaited(
        widget.controller.moveActiveSheetCell(rowDelta: 0, columnDelta: 1),
      );
      return KeyEventResult.handled;
    }
    if (event.logicalKey == LogicalKeyboardKey.arrowUp) {
      unawaited(
        widget.controller.moveActiveSheetCell(rowDelta: -1, columnDelta: 0),
      );
      return KeyEventResult.handled;
    }
    if (event.logicalKey == LogicalKeyboardKey.arrowDown) {
      unawaited(
        widget.controller.moveActiveSheetCell(rowDelta: 1, columnDelta: 0),
      );
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  void _submitSearch(String query) {
    unawaited(widget.controller.search(query));
  }

  Widget _buildPasswordPrompt(BuildContext context) {
    final passwordState = widget.controller.passwordPromptState;
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            passwordState.message ??
                'Password is required to open this document.',
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _passwordController,
            obscureText: true,
            decoration: const InputDecoration(
              border: OutlineInputBorder(),
              labelText: 'Document password',
            ),
          ),
          const SizedBox(height: 12),
          ElevatedButton(
            onPressed: () {
              widget.controller.submitPassword(_passwordController.text);
            },
            child: const Text('Open document'),
          ),
        ],
      ),
    );
  }
}
