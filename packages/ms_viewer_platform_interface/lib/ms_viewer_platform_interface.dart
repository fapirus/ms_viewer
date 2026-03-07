import 'package:plugin_platform_interface/plugin_platform_interface.dart';

import 'src/models/open_document_contract.dart';

export 'src/models/open_document_contract.dart';
export 'src/models/render_model.dart';

abstract class MsViewerPlatform extends PlatformInterface {
  MsViewerPlatform() : super(token: _token);

  static final Object _token = Object();

  static MsViewerPlatform _instance = _NoopMsViewerPlatform();

  static MsViewerPlatform get instance => _instance;

  static set instance(MsViewerPlatform instance) {
    PlatformInterface.verifyToken(instance, _token);
    _instance = instance;
  }

  Future<OpenDocumentResult> openDocument(OpenDocumentRequest request) async {
    return OpenDocumentFailure(
      const OpenDocumentError(
        code: ViewerErrorCode.notImplemented,
        message: 'openDocument is not implemented.',
      ),
    );
  }

  Future<GetPageRenderModelResult> getPageRenderModel(
    GetPageRenderModelRequest request,
  ) async {
    return const GetPageRenderModelFailure(
      OpenDocumentError(
        code: ViewerErrorCode.notImplemented,
        message: 'getPageRenderModel is not implemented.',
      ),
    );
  }

  Future<SearchDocumentResult> searchDocument(
    SearchDocumentRequest request,
  ) async {
    return const SearchDocumentFailure(
      OpenDocumentError(
        code: ViewerErrorCode.notImplemented,
        message: 'searchDocument is not implemented.',
      ),
    );
  }

  Future<GetSelectionPageResult> getSelectionPage(
    GetSelectionPageRequest request,
  ) async {
    return const GetSelectionPageFailure(
      OpenDocumentError(
        code: ViewerErrorCode.notImplemented,
        message: 'getSelectionPage is not implemented.',
      ),
    );
  }
}

class _NoopMsViewerPlatform extends MsViewerPlatform {}
