import 'package:flutter/material.dart';
import 'package:ms_viewer/ms_viewer.dart';

void main() {
  runApp(const DemoApp());
}

class DemoApp extends StatelessWidget {
  const DemoApp({super.key});

  @override
  Widget build(BuildContext context) {
    final controller = MsViewerController(
      document: const DocumentDescriptor(
        id: 'demo-docx',
        kind: DocumentKind.docx,
        title: 'Demo DOCX',
        pageCount: 12,
      ),
    );

    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('MS Viewer Demo')),
        body: MsDocumentView(controller: controller),
      ),
    );
  }
}
