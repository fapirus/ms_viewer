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
