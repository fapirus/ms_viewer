import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_demo/main.dart';

void main() {
  testWidgets('demo app renders fixture browser shell', (tester) async {
    await tester.pumpWidget(const DemoApp());
    await tester.pumpAndSettle();

    expect(find.text('MS Viewer Demo'), findsOneWidget);
    expect(find.text('Fixtures'), findsOneWidget);
    expect(find.text('Open File'), findsOneWidget);
    expect(find.text('docx_plain_text.docx'), findsWidgets);
    expect(find.text('Bundled fixture'), findsWidgets);
  });
}
