# Windows notification support

This build adds a background Windows Notification Listener to Nock.

## Behavior

- Requests access to Windows notification-center data when Nock starts.
- Ignores notifications already present at startup and surfaces newly detected toast notifications.
- Shows the application identifier and notification title in the collapsed pill for about 6.5 seconds.
- When the island is expanded, shows the title and message body in a notification card.
- Uses Nock's existing Win32 message loop; notification polling runs on a background thread.

## Permission

If Windows denies access, Nock continues to run and logs the denial to stderr. Availability of `UserNotificationListener` can vary by Windows build and application identity. If Windows does not grant access to this unpackaged portable build, this implementation cannot bypass that restriction.

## Build on Windows

Install the stable Rust toolchain with the MSVC target and Windows build tools, then run `build.bat` from this folder. The build has not been executed in the authoring environment because no Rust toolchain is installed there; please run the build and report any compiler diagnostics before treating the binary as verified.
