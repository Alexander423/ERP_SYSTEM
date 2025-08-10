use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Base trait for email templates
pub trait EmailTemplate: Send + Sync {
    /// Get the email subject
    fn subject(&self) -> String;
    
    /// Get the HTML body
    fn html_body(&self) -> String;
    
    /// Get the text body (fallback)
    fn text_body(&self) -> String;
    
    /// Get template name for logging/debugging
    fn template_name(&self) -> &'static str;
}

/// Email verification template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationEmailTemplate {
    pub user_name: String,
    pub company_name: String,
    pub verification_url: String,
    pub expires_in_hours: u32,
}

impl EmailTemplate for VerificationEmailTemplate {
    fn subject(&self) -> String {
        format!("Verify your account for {}", self.company_name)
    }

    fn html_body(&self) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Email Verification</title>
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background-color: #2563eb; color: white; padding: 20px; text-align: center; }}
        .content {{ padding: 20px; background-color: #f8fafc; }}
        .button {{ 
            display: inline-block; 
            background-color: #2563eb; 
            color: white; 
            padding: 12px 24px; 
            text-decoration: none; 
            border-radius: 6px; 
            margin: 20px 0; 
        }}
        .footer {{ padding: 20px; text-align: center; color: #6b7280; font-size: 14px; }}
        .warning {{ color: #dc2626; font-weight: bold; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Welcome to {}</h1>
        </div>
        <div class="content">
            <h2>Hi {},</h2>
            <p>Thank you for registering with {}! To complete your account setup, please verify your email address by clicking the button below.</p>
            
            <div style="text-align: center;">
                <a href="{}" class="button">Verify Email Address</a>
            </div>
            
            <p><strong>This verification link will expire in {} hours.</strong></p>
            
            <p>If you didn't create an account with us, you can safely ignore this email.</p>
            
            <p>If you're unable to click the button above, copy and paste the following link into your browser:</p>
            <p style="word-break: break-all; color: #2563eb;">{}</p>
        </div>
        <div class="footer">
            <p>This is an automated email. Please do not reply to this message.</p>
            <p>&copy; {} ERP System. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
            "#,
            self.company_name,
            self.user_name,
            self.company_name,
            self.verification_url,
            self.expires_in_hours,
            self.verification_url,
            self.company_name
        )
    }

    fn text_body(&self) -> String {
        format!(
            r#"
Welcome to {}!

Hi {},

Thank you for registering with {}! To complete your account setup, please verify your email address by visiting the following link:

{}

This verification link will expire in {} hours.

If you didn't create an account with us, you can safely ignore this email.

---
This is an automated email. Please do not reply to this message.
© {} ERP System. All rights reserved.
            "#,
            self.company_name,
            self.user_name,
            self.company_name,
            self.verification_url,
            self.expires_in_hours,
            self.company_name
        ).trim().to_string()
    }

    fn template_name(&self) -> &'static str {
        "email_verification"
    }
}

/// Password reset email template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetEmailTemplate {
    pub user_name: String,
    pub company_name: String,
    pub reset_url: String,
    pub expires_in_hours: u32,
    pub source_ip: Option<String>,
}

impl EmailTemplate for PasswordResetEmailTemplate {
    fn subject(&self) -> String {
        format!("Password reset request for {}", self.company_name)
    }

    fn html_body(&self) -> String {
        let ip_info = if let Some(ip) = &self.source_ip {
            format!("<p><strong>Request origin:</strong> {}</p>", ip)
        } else {
            String::new()
        };

        format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Password Reset</title>
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background-color: #dc2626; color: white; padding: 20px; text-align: center; }}
        .content {{ padding: 20px; background-color: #f8fafc; }}
        .button {{ 
            display: inline-block; 
            background-color: #dc2626; 
            color: white; 
            padding: 12px 24px; 
            text-decoration: none; 
            border-radius: 6px; 
            margin: 20px 0; 
        }}
        .footer {{ padding: 20px; text-align: center; color: #6b7280; font-size: 14px; }}
        .warning {{ color: #dc2626; font-weight: bold; }}
        .security-info {{ background-color: #fef2f2; border: 1px solid #fecaca; padding: 15px; margin: 15px 0; border-radius: 6px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Password Reset Request</h1>
        </div>
        <div class="content">
            <h2>Hi {},</h2>
            <p>We received a request to reset your password for your {} account.</p>
            
            <div class="security-info">
                <p><strong>Security Information:</strong></p>
                {}
                <p><strong>Request time:</strong> Just now</p>
                <p class="warning">If you didn't request this password reset, please contact support immediately.</p>
            </div>
            
            <div style="text-align: center;">
                <a href="{}" class="button">Reset Password</a>
            </div>
            
            <p><strong>This password reset link will expire in {} hours.</strong></p>
            
            <p>If you didn't request a password reset, you can safely ignore this email. Your password will not be changed.</p>
            
            <p>If you're unable to click the button above, copy and paste the following link into your browser:</p>
            <p style="word-break: break-all; color: #dc2626;">{}</p>
        </div>
        <div class="footer">
            <p>This is an automated email. Please do not reply to this message.</p>
            <p>If you need help, contact our support team.</p>
            <p>&copy; {} ERP System. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
            "#,
            self.user_name,
            self.company_name,
            ip_info,
            self.reset_url,
            self.expires_in_hours,
            self.reset_url,
            self.company_name
        )
    }

    fn text_body(&self) -> String {
        let ip_info = if let Some(ip) = &self.source_ip {
            format!("Request origin: {}\n", ip)
        } else {
            String::new()
        };

        format!(
            r#"
Password Reset Request

Hi {},

We received a request to reset your password for your {} account.

Security Information:
{}Request time: Just now

WARNING: If you didn't request this password reset, please contact support immediately.

To reset your password, visit the following link:
{}

This password reset link will expire in {} hours.

If you didn't request a password reset, you can safely ignore this email. Your password will not be changed.

---
This is an automated email. Please do not reply to this message.
If you need help, contact our support team.
© {} ERP System. All rights reserved.
            "#,
            self.user_name,
            self.company_name,
            ip_info,
            self.reset_url,
            self.expires_in_hours,
            self.company_name
        ).trim().to_string()
    }

    fn template_name(&self) -> &'static str {
        "password_reset"
    }
}

/// Welcome email template (after successful verification)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelcomeEmailTemplate {
    pub user_name: String,
    pub company_name: String,
    pub login_url: String,
}

impl EmailTemplate for WelcomeEmailTemplate {
    fn subject(&self) -> String {
        format!("Welcome to {}! Your account is now active", self.company_name)
    }

    fn html_body(&self) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Welcome</title>
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background-color: #16a34a; color: white; padding: 20px; text-align: center; }}
        .content {{ padding: 20px; background-color: #f8fafc; }}
        .button {{ 
            display: inline-block; 
            background-color: #16a34a; 
            color: white; 
            padding: 12px 24px; 
            text-decoration: none; 
            border-radius: 6px; 
            margin: 20px 0; 
        }}
        .footer {{ padding: 20px; text-align: center; color: #6b7280; font-size: 14px; }}
        .features {{ background-color: #f0fdf4; border: 1px solid #bbf7d0; padding: 15px; margin: 15px 0; border-radius: 6px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🎉 Welcome to {}!</h1>
        </div>
        <div class="content">
            <h2>Hi {},</h2>
            <p>Congratulations! Your email has been verified and your account is now active.</p>
            
            <div class="features">
                <h3>What you can do now:</h3>
                <ul>
                    <li>Access your company dashboard</li>
                    <li>Manage users and permissions</li>
                    <li>Configure system settings</li>
                    <li>Start using all ERP features</li>
                </ul>
            </div>
            
            <div style="text-align: center;">
                <a href="{}" class="button">Access Your Account</a>
            </div>
            
            <p>If you have any questions or need assistance getting started, our support team is here to help.</p>
        </div>
        <div class="footer">
            <p>Thank you for choosing our ERP system!</p>
            <p>&copy; {} ERP System. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
            "#,
            self.company_name,
            self.user_name,
            self.login_url,
            self.company_name
        )
    }

    fn text_body(&self) -> String {
        format!(
            r#"
🎉 Welcome to {}!

Hi {},

Congratulations! Your email has been verified and your account is now active.

What you can do now:
- Access your company dashboard
- Manage users and permissions  
- Configure system settings
- Start using all ERP features

Access your account: {}

If you have any questions or need assistance getting started, our support team is here to help.

Thank you for choosing our ERP system!
© {} ERP System. All rights reserved.
            "#,
            self.company_name,
            self.user_name,
            self.login_url,
            self.company_name
        ).trim().to_string()
    }

    fn template_name(&self) -> &'static str {
        "welcome"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_email_template() {
        let template = VerificationEmailTemplate {
            user_name: "John Doe".to_string(),
            company_name: "Acme Corp".to_string(),
            verification_url: "https://example.com/verify?token=abc123".to_string(),
            expires_in_hours: 24,
        };

        let subject = template.subject();
        assert!(subject.contains("Acme Corp"));
        
        let html = template.html_body();
        assert!(html.contains("John Doe"));
        assert!(html.contains("https://example.com/verify?token=abc123"));
        assert!(html.contains("24 hours"));
        
        let text = template.text_body();
        assert!(text.contains("John Doe"));
        assert!(text.contains("Acme Corp"));
    }

    #[test]
    fn test_password_reset_template() {
        let template = PasswordResetEmailTemplate {
            user_name: "Jane Smith".to_string(),
            company_name: "Test Company".to_string(),
            reset_url: "https://example.com/reset?token=xyz789".to_string(),
            expires_in_hours: 1,
            source_ip: Some("192.168.1.1".to_string()),
        };

        let subject = template.subject();
        assert!(subject.contains("Test Company"));
        
        let html = template.html_body();
        assert!(html.contains("Jane Smith"));
        assert!(html.contains("192.168.1.1"));
        assert!(html.contains("1 hours"));
    }
}