import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

void main() {
  test('default platform exists', () {
    expect(MsViewerPlatform.instance, isNotNull);
  });
}
