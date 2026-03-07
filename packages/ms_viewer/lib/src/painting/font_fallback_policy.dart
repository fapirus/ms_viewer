enum ViewerPlatform { windows, macOs, android, ios }

enum ScriptKind { latin, cjk }

String resolveFontFamily({
  required ViewerPlatform platform,
  required String requested,
  required ScriptKind script,
}) {
  switch ((platform, requested)) {
    case (ViewerPlatform.windows, 'Calibri'):
      return 'Arial';
    case (ViewerPlatform.windows, 'Cambria'):
      return 'Times New Roman';
    case (ViewerPlatform.windows, 'Malgun Gothic'):
      return 'Malgun Gothic';
    case (ViewerPlatform.windows, 'Batang'):
      return 'Malgun Gothic';
    case (ViewerPlatform.macOs, 'Calibri'):
      return 'Helvetica';
    case (ViewerPlatform.macOs, 'Cambria'):
      return 'Times';
    case (ViewerPlatform.macOs, 'Malgun Gothic'):
      return 'Apple SD Gothic Neo';
    case (ViewerPlatform.macOs, 'Batang'):
      return 'Apple SD Gothic Neo';
    case (ViewerPlatform.android, 'Calibri'):
      return 'Roboto';
    case (ViewerPlatform.android, 'Cambria'):
      return 'Noto Serif';
    case (ViewerPlatform.android, 'Malgun Gothic'):
      return 'Noto Sans CJK KR';
    case (ViewerPlatform.android, 'Batang'):
      return 'Noto Sans CJK KR';
    case (ViewerPlatform.ios, 'Calibri'):
      return 'Helvetica';
    case (ViewerPlatform.ios, 'Cambria'):
      return 'Times New Roman';
    case (ViewerPlatform.ios, 'Malgun Gothic'):
      return 'Apple SD Gothic Neo';
    case (ViewerPlatform.ios, 'Batang'):
      return 'Apple SD Gothic Neo';
    default:
      return _genericFallback(platform: platform, script: script);
  }
}

String resolveRenderableFontFamily({
  required ViewerPlatform platform,
  required String? requested,
  required ScriptKind script,
}) {
  final normalized = requested?.trim();
  if (normalized == null || normalized.isEmpty) {
    return _genericFallback(platform: platform, script: script);
  }

  final mapped = _mappedFontFamily(
    platform: platform,
    requested: normalized,
  );
  return mapped ?? normalized;
}

List<String> resolveFontFamilyFallbacks({
  required ViewerPlatform platform,
  required String? requested,
  required ScriptKind script,
}) {
  final generic = _genericFallback(platform: platform, script: script);
  final mapped = requested == null || requested.trim().isEmpty
      ? null
      : _mappedFontFamily(platform: platform, requested: requested.trim());

  if (mapped == null || mapped == generic) {
    return [generic];
  }

  return [mapped, generic];
}

ScriptKind scriptKindForText(String text) {
  for (final rune in text.runes) {
    if (_isWideCodePoint(rune)) {
      return ScriptKind.cjk;
    }
  }

  return ScriptKind.latin;
}

String? _mappedFontFamily({
  required ViewerPlatform platform,
  required String requested,
}) {
  switch ((platform, requested)) {
    case (ViewerPlatform.windows, 'Calibri'):
    case (ViewerPlatform.windows, 'Aptos'):
      return 'Arial';
    case (ViewerPlatform.windows, 'Cambria'):
      return 'Times New Roman';
    case (ViewerPlatform.windows, 'Malgun Gothic'):
    case (ViewerPlatform.windows, '맑은 고딕'):
      return 'Malgun Gothic';
    case (ViewerPlatform.windows, 'Batang'):
    case (ViewerPlatform.windows, '바탕'):
      return 'Malgun Gothic';
    case (ViewerPlatform.windows, 'Dotum'):
    case (ViewerPlatform.windows, '돋움'):
    case (ViewerPlatform.windows, 'Gulim'):
    case (ViewerPlatform.windows, '굴림'):
    case (ViewerPlatform.windows, 'Nanum Gothic'):
      return 'Malgun Gothic';
    case (ViewerPlatform.macOs, 'Calibri'):
    case (ViewerPlatform.macOs, 'Aptos'):
      return 'Helvetica';
    case (ViewerPlatform.macOs, 'Cambria'):
      return 'Times';
    case (ViewerPlatform.macOs, 'Malgun Gothic'):
    case (ViewerPlatform.macOs, '맑은 고딕'):
    case (ViewerPlatform.macOs, 'Batang'):
    case (ViewerPlatform.macOs, '바탕'):
    case (ViewerPlatform.macOs, 'Dotum'):
    case (ViewerPlatform.macOs, '돋움'):
    case (ViewerPlatform.macOs, 'Gulim'):
    case (ViewerPlatform.macOs, '굴림'):
    case (ViewerPlatform.macOs, 'Nanum Gothic'):
      return 'Apple SD Gothic Neo';
    case (ViewerPlatform.android, 'Calibri'):
    case (ViewerPlatform.android, 'Aptos'):
      return 'Roboto';
    case (ViewerPlatform.android, 'Cambria'):
      return 'Noto Serif';
    case (ViewerPlatform.android, 'Malgun Gothic'):
    case (ViewerPlatform.android, '맑은 고딕'):
    case (ViewerPlatform.android, 'Batang'):
    case (ViewerPlatform.android, '바탕'):
    case (ViewerPlatform.android, 'Dotum'):
    case (ViewerPlatform.android, '돋움'):
    case (ViewerPlatform.android, 'Gulim'):
    case (ViewerPlatform.android, '굴림'):
    case (ViewerPlatform.android, 'Nanum Gothic'):
      return 'Noto Sans CJK KR';
    case (ViewerPlatform.ios, 'Calibri'):
    case (ViewerPlatform.ios, 'Aptos'):
      return 'Helvetica';
    case (ViewerPlatform.ios, 'Cambria'):
      return 'Times New Roman';
    case (ViewerPlatform.ios, 'Malgun Gothic'):
    case (ViewerPlatform.ios, '맑은 고딕'):
    case (ViewerPlatform.ios, 'Batang'):
    case (ViewerPlatform.ios, '바탕'):
    case (ViewerPlatform.ios, 'Dotum'):
    case (ViewerPlatform.ios, '돋움'):
    case (ViewerPlatform.ios, 'Gulim'):
    case (ViewerPlatform.ios, '굴림'):
    case (ViewerPlatform.ios, 'Nanum Gothic'):
      return 'Apple SD Gothic Neo';
    default:
      return null;
  }
}

String _genericFallback({
  required ViewerPlatform platform,
  required ScriptKind script,
}) {
  switch ((platform, script)) {
    case (ViewerPlatform.windows, ScriptKind.latin):
      return 'Arial';
    case (ViewerPlatform.windows, ScriptKind.cjk):
      return 'Malgun Gothic';
    case (ViewerPlatform.macOs, ScriptKind.latin):
      return 'Helvetica';
    case (ViewerPlatform.macOs, ScriptKind.cjk):
      return 'Apple SD Gothic Neo';
    case (ViewerPlatform.android, ScriptKind.latin):
      return 'Roboto';
    case (ViewerPlatform.android, ScriptKind.cjk):
      return 'Noto Sans CJK KR';
    case (ViewerPlatform.ios, ScriptKind.latin):
      return 'Helvetica';
    case (ViewerPlatform.ios, ScriptKind.cjk):
      return 'Apple SD Gothic Neo';
  }
}

bool _isWideCodePoint(int codePoint) {
  return (codePoint >= 0x1100 && codePoint <= 0x11FF) ||
      (codePoint >= 0x2E80 && codePoint <= 0xA4CF) ||
      (codePoint >= 0xAC00 && codePoint <= 0xD7AF) ||
      (codePoint >= 0xF900 && codePoint <= 0xFAFF) ||
      (codePoint >= 0xFE10 && codePoint <= 0xFE6F) ||
      (codePoint >= 0xFF01 && codePoint <= 0xFF60) ||
      (codePoint >= 0xFFE0 && codePoint <= 0xFFE6) ||
      (codePoint >= 0x1F300 && codePoint <= 0x1FAFF);
}
