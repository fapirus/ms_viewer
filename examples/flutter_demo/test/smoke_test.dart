import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_demo/main.dart';

void main() {
  testWidgets('demo app renders shell', (tester) async {
    await tester.pumpWidget(const DemoApp());

    expect(find.text('MS Viewer Demo'), findsOneWidget);
    expect(find.text('Demo DOCX'), findsOneWidget);
    expect(find.text('12 pages'), findsOneWidget);
    expect(find.textContaining('render model not loaded'), findsOneWidget);
  });
}
