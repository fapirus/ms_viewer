import 'package:flutter_test/flutter_test.dart';
import 'package:ms_viewer/ms_viewer.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

void main() {
  test('platform password required error maps to password prompt state', () {
    const error = OpenDocumentError(
      code: ViewerErrorCode.passwordRequired,
      message: 'Password is required.',
    );
    const state = PasswordPromptState(status: PasswordPromptStatus.idle);

    final next = error.toPasswordPromptState(state);

    expect(next, isNotNull);
    expect(next!.status, PasswordPromptStatus.awaitingPassword);
    expect(next.message, 'Password is required.');
  });

  test('invalid password mapping increments attempt count', () {
    const error = OpenDocumentError(
      code: ViewerErrorCode.invalidPassword,
      message: 'Wrong password.',
    );
    const state = PasswordPromptState(
      status: PasswordPromptStatus.awaitingPassword,
      attemptCount: 1,
    );

    final next = error.toPasswordPromptState(state);

    expect(next, isNotNull);
    expect(next!.status, PasswordPromptStatus.invalidPassword);
    expect(next.attemptCount, 2);
  });

  test('non-password errors map to viewer exception', () {
    const error = OpenDocumentError(
      code: ViewerErrorCode.invalidDocument,
      message: 'Broken OOXML package.',
    );

    final exception = error.toViewerException();

    expect(exception.kind, MsViewerFailureKind.invalidDocument);
    expect(exception.message, 'Broken OOXML package.');
  });

  test('non-password errors do not force password prompt state', () {
    const error = OpenDocumentError(
      code: ViewerErrorCode.ioError,
      message: 'Disk read failed.',
    );
    const state = PasswordPromptState(status: PasswordPromptStatus.idle);

    final next = error.toPasswordPromptState(state);

    expect(next, isNull);
  });
}
