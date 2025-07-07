# Security Policy

## Supported Versions

Currently, only the latest release of steamdeck-Controls is supported with security updates.

| Version | Supported          |
| ------- | ------------------ |
| Latest  | :white_check_mark: |
| Older   | :x:                |

## Security Considerations

### Network Security
- **WebSocket Server**: This application runs a WebSocket server that listens for connections. By default, it uses port 8080.
- **Local Network Only**: This software is designed for use on local networks only. Do not expose the WebSocket server to the internet.
- **Firewall**: Consider configuring your firewall to restrict access to the WebSocket port to trusted devices only.

### Known Limitations
- No authentication mechanism is implemented for WebSocket connections
- No encryption is used for data transmission between SteamDeck and PC
- The server accepts any connection from the local network

## Reporting a Vulnerability

If you discover a security vulnerability in steamdeck-Controls, please report it responsibly:

1. **Do NOT** create a public GitHub issue for security vulnerabilities
2. Send an email to the repository owner or create a private security advisory on GitHub
3. Include the following information:
   - Description of the vulnerability
   - Steps to reproduce the issue
   - Potential impact
   - Suggested fix (if you have one)

### Response Timeline
- **Acknowledgment**: Within 48 hours of report
- **Initial Assessment**: Within 1 week
- **Fix Timeline**: Depends on severity and complexity

**Note**: As stated in the project documentation, ongoing support and development are not guaranteed. Security fixes will be evaluated on a case-by-case basis.

## Security Best Practices for Users

### Network Configuration
- Use the application only on trusted local networks
- Consider changing the default port (8080) if it conflicts with other services
- Monitor network traffic if you have concerns about data transmission

### General Security
- Keep VIGEmBus updated to the latest version
- Regularly update your operating system and security software
- Review the source code before use if you have security concerns

## Disclaimer

This software is provided under the GPL-3.0 license "as is" without warranty of any kind. Users assume all risks associated with using this software, including but not limited to:

- Unauthorized network access through the WebSocket server
- Data interception on unsecured networks
- Potential system vulnerabilities

The maintainer is not responsible for any security incidents, data breaches, or system compromises that may result from using this software.

## License

This security policy is part of the steamdeck-Controls project and is subject to the same GPL-3.0 license terms.
