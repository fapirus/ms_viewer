import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer/ms_viewer.dart';

void main() {
  test('invalid password increments attempt count', () {
    const state = PasswordPromptState(
      status: PasswordPromptStatus.awaitingPassword,
    );

    final next = state.invalidPassword('Wrong password');

    expect(next.status, PasswordPromptStatus.invalidPassword);
    expect(next.attemptCount, 1);
    expect(next.message, 'Wrong password');
  });

  test('cancelled state is preserved explicitly', () {
    const state = PasswordPromptState(
      status: PasswordPromptStatus.awaitingPassword,
    );

    final cancelled = state.cancelled();

    expect(cancelled.status, PasswordPromptStatus.cancelled);
  });
}
