enum PasswordPromptStatus {
  idle,
  awaitingPassword,
  invalidPassword,
  unsupportedEncryption,
  cancelled,
  success,
}

class PasswordPromptState {
  const PasswordPromptState({
    required this.status,
    this.message,
    this.attemptCount = 0,
  });

  final PasswordPromptStatus status;
  final String? message;
  final int attemptCount;

  PasswordPromptState requirePassword([String? message]) {
    return PasswordPromptState(
      status: PasswordPromptStatus.awaitingPassword,
      message: message,
      attemptCount: attemptCount,
    );
  }

  PasswordPromptState invalidPassword([String? message]) {
    return PasswordPromptState(
      status: PasswordPromptStatus.invalidPassword,
      message: message,
      attemptCount: attemptCount + 1,
    );
  }

  PasswordPromptState unsupportedEncryption([String? message]) {
    return PasswordPromptState(
      status: PasswordPromptStatus.unsupportedEncryption,
      message: message,
      attemptCount: attemptCount,
    );
  }

  PasswordPromptState cancelled() {
    return PasswordPromptState(
      status: PasswordPromptStatus.cancelled,
      message: message,
      attemptCount: attemptCount,
    );
  }

  PasswordPromptState success() {
    return PasswordPromptState(
      status: PasswordPromptStatus.success,
      message: null,
      attemptCount: attemptCount,
    );
  }
}
