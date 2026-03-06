import 'package:flutter/foundation.dart';

import '../models/document_descriptor.dart';

class MsViewerController extends ChangeNotifier {
  MsViewerController({this.document});

  DocumentDescriptor? document;

  void attachDocument(DocumentDescriptor next) {
    document = next;
    notifyListeners();
  }
}
