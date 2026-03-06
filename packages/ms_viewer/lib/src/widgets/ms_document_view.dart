import 'package:flutter/material.dart';

import '../controller/ms_viewer_controller.dart';

class MsDocumentView extends StatefulWidget {
  const MsDocumentView({super.key, required this.controller});

  final MsViewerController controller;

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
        return Center(
          child: Text(
            'Viewer placeholder for ${document.title} (${document.pageCount} pages)',
            textAlign: TextAlign.center,
          ),
        );
      case ViewerShellStatus.idle:
        final document = widget.controller.document;
        if (document == null) {
          return const Center(child: Text('No document attached'));
        }
        return Center(
          child: Text(
            'Viewer placeholder for ${document.title} (${document.pageCount} pages)',
            textAlign: TextAlign.center,
          ),
        );
    }
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
