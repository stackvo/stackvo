:80 {
    root * /var/www/html/{{DOCUMENT_ROOT}}
    
    # Enable PHP-FPM (localhost - same container)
    php_fastcgi 127.0.0.1:9000
    
    # Enable file server
    file_server
    
    # Logging
    log {
        output stdout
        format console
    }
}
