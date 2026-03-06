import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

import '../password/password_prompt_state.dart';

enum MsViewerFailureKind {
  unsupportedFormat,
  passwordRequired,
  invalidPassword,
  unsupportedEncryption,
  ioError,
  invalidDocument,
  notImplemented,
}

class MsViewerException implements Exception {
  const MsViewerException({required this.kind, required this.message});

  final MsViewerFailureKind kind;
  final String message;

  @override
  String toString() => 'MsViewerException($kind): $message';
}

extension OpenDocumentErrorMapping on OpenDocumentError {
  MsViewerException toViewerException() {
    return MsViewerException(
      kind: switch (code) {
        ViewerErrorCode.unsupportedFormat =>
          MsViewerFailureKind.unsupportedFormat,
        ViewerErrorCode.passwordRequired =>
          MsViewerFailureKind.passwordRequired,
        ViewerErrorCode.invalidPassword =>
          MsViewerFailureKind.invalidPassword,
        ViewerErrorCode.unsupportedEncryption =>
          MsViewerFailureKind.unsupportedEncryption,
        ViewerErrorCode.ioError => MsViewerFailureKind.ioError,
        ViewerErrorCode.invalidDocument =>
          MsViewerFailureKind.invalidDocument,
        ViewerErrorCode.notImplemented =>
          MsViewerFailureKind.notImplemented,
      },
      message: message,
    );
  }

  PasswordPromptState? toPasswordPromptState(PasswordPromptState current) {
    return switch (code) {
      ViewerErrorCode.passwordRequired => current.requirePassword(message),
      ViewerErrorCode.invalidPassword => current.invalidPassword(message),
      ViewerErrorCode.unsupportedEncryption =>
        current.unsupportedEncryption(message),
      _ => null,
    };
  }
}
