import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer/ms_viewer.dart';

void main() {
  test('uses explicit platform font mapping first', () {
    expect(
      resolveFontFamily(
        platform: ViewerPlatform.windows,
        requested: 'Calibri',
        script: ScriptKind.latin,
      ),
      'Arial',
    );
  });

  test('falls back to script default for unknown fonts', () {
    expect(
      resolveFontFamily(
        platform: ViewerPlatform.macOs,
        requested: 'Unknown Font',
        script: ScriptKind.cjk,
      ),
      'Apple SD Gothic Neo',
    );
  });
}
