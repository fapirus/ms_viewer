import 'package:flutter/material.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

import '../controller/ms_viewer_controller.dart';
import 'document_page_view.dart';

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

  @override
  void initState() {
    super.initState();
    _passwordController = TextEditingController();
  }

  @override
  void dispose() {
    _passwordController.dispose();
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
        return _buildReadyState(document.title, document.pageCount);
      case ViewerShellStatus.idle:
        final document = widget.controller.document;
        if (document == null) {
          return const Center(child: Text('No document attached'));
        }
        return _buildReadyState(document.title, document.pageCount);
    }
  }

  Widget _buildReadyState(String title, int pageCount) {
    final fetchedPage = widget.controller.currentPage;
    final fallbackPreviewPage = widget.previewPages.isEmpty
        ? null
        : widget.previewPages.first;
    final page = fetchedPage ?? fallbackPreviewPage;
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            title,
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const SizedBox(height: 8),
          Text('$pageCount pages'),
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
              ViewerPageStatus.ready when page != null => Center(
                  child: ConstrainedBox(
                    constraints: const BoxConstraints(maxWidth: 640),
                    child: DocumentPageView(page: page),
                  ),
                ),
              _ when page != null => Center(
                  child: ConstrainedBox(
                    constraints: const BoxConstraints(maxWidth: 640),
                    child: DocumentPageView(page: page),
                  ),
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

  Widget _buildPasswordPrompt(BuildContext context) {
    final passwordState = widget.controller.passwordPromptState;
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            passwordState.message ?? 'Password is required to open this document.',
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
