import 'package:plugin_platform_interface/plugin_platform_interface.dart';

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
}

class _NoopMsViewerPlatform extends MsViewerPlatform {}
