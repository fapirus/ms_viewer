import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer/ms_viewer.dart';

void main() {
  testWidgets('search result list widget renders page previews', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SearchResultList(
            results: const [
              SearchResult(
                query: 'alpha',
                pageIndex: 0,
                start: 0,
                end: 5,
                preview: 'alpha preview',
              ),
              SearchResult(
                query: 'alpha',
                pageIndex: 2,
                start: 10,
                end: 15,
                preview: 'second match',
              ),
            ],
            currentIndex: 1,
          ),
        ),
      ),
    );

    expect(find.text('Page 1'), findsOneWidget);
    expect(find.text('Page 3'), findsOneWidget);
    expect(find.text('alpha preview'), findsOneWidget);
    expect(find.text('second match'), findsOneWidget);
  });
}
