import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_demo/main.dart';

void main() {
  testWidgets('demo app renders shell', (tester) async {
    await tester.pumpWidget(const DemoApp());

    expect(find.text('MS Viewer Demo'), findsOneWidget);
    expect(find.textContaining('Viewer placeholder for Demo DOCX'), findsOneWidget);
  });
}
