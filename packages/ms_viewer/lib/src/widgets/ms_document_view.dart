import 'package:flutter/material.dart';

import '../controller/ms_viewer_controller.dart';

class MsDocumentView extends StatelessWidget {
  const MsDocumentView({super.key, required this.controller});

  final MsViewerController controller;

  @override
  Widget build(BuildContext context) {
    final document = controller.document;
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
