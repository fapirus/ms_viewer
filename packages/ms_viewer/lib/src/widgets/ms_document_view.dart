import 'dart:async';

import 'package:flutter/material.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

import '../controller/ms_viewer_controller.dart';
import '../models/document_descriptor.dart' as document_model;
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

  @override
  void initState() {
    super.initState();
    _passwordController = TextEditingController();
    _searchController = TextEditingController();
  }

  @override
  void dispose() {
    _passwordController.dispose();
    _searchController.dispose();
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
    final collectionLabel = isSpreadsheet ? 'sheets' : 'pages';
    final currentLabel = isSpreadsheet ? 'Sheet' : 'Page';
    final fetchedPage = widget.controller.currentPage;
    final fallbackPreviewPage = widget.previewPages.isEmpty
        ? null
        : widget.previewPages.first;
    final page = fetchedPage ?? fallbackPreviewPage;
    final searchResults = widget.controller.searchController.results;
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(title, style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 8),
          Text('$pageCount $collectionLabel'),
          const SizedBox(height: 16),
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
            child: switch (widget.controller.pageStatus) {
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
              ViewerPageStatus.ready when page != null => _buildViewerSurface(
                page,
                isSpreadsheet,
              ),
              _ when page != null => _buildViewerSurface(page, isSpreadsheet),
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

  Widget _buildViewerSurface(PageRenderModel page, bool isSpreadsheet) {
    final viewer = isSpreadsheet
        ? SheetViewport(
            page: page,
            highlights: widget.controller.pageHighlights,
            onSelectionStart: widget.controller.startSelectionAt,
            onSelectionUpdate: widget.controller.updateSelectionAt,
          )
        : DocumentPageView(
            page: page,
            highlights: widget.controller.pageHighlights,
            onSelectionStart: widget.controller.startSelectionAt,
            onSelectionUpdate: widget.controller.updateSelectionAt,
          );

    if (isSpreadsheet) {
      return viewer;
    }

    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 640),
        child: viewer,
      ),
    );
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
