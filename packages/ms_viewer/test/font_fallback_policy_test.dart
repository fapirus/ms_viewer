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

  test('renderable font keeps unknown requested family when possible', () {
    expect(
      resolveRenderableFontFamily(
        platform: ViewerPlatform.macOs,
        requested: 'Pretend Custom Sans',
        script: ScriptKind.latin,
      ),
      'Pretend Custom Sans',
    );
  });

  test('script detection treats Hangul as cjk', () {
    expect(scriptKindForText('안녕하세요'), ScriptKind.cjk);
    expect(scriptKindForText('hello world'), ScriptKind.latin);
  });

  test('fallback chain includes platform generic for mapped fonts', () {
    expect(
      resolveFontFamilyFallbacks(
        platform: ViewerPlatform.macOs,
        requested: '맑은 고딕',
        script: ScriptKind.cjk,
      ),
      ['Apple SD Gothic Neo'],
    );
  });
}
