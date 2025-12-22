worker_processes 1;

events {
    worker_connections 1024;
}

http {
    include       /etc/nginx/mime.types;
    default_type  application/octet-stream;

    # Healthcheck server (always enabled)
    server {
        listen 80 default_server;
        server_name _;
        
        location = /health {
            access_log off;
            return 200 "OK";
        }
        
        location / {
            return 404 "Not Found";
        }
    }

{{#TOOLS_PHPMYADMIN_ENABLE}}
    # PHPMyAdmin
    server {
        listen 80;
        server_name phpmyadmin.*;
        
        root /var/www/html/phpmyadmin;
        index index.php;
        
        location / {
            try_files $uri $uri/ /index.php?$query_string;
        }
        
        location ~ \.php$ {
            fastcgi_pass 127.0.0.1:9000;
            fastcgi_index index.php;
            include fastcgi_params;
            fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        }
    }

{{/TOOLS_PHPMYADMIN_ENABLE}}
{{#TOOLS_ADMINER_ENABLE}}
    # Adminer
    server {
        listen 80;
        server_name adminer.*;
        
        root /var/www/html/adminer;
        index index.php;
        
        location / {
            try_files $uri $uri/ /index.php?$query_string;
        }
        
        location ~ \.php$ {
            fastcgi_pass 127.0.0.1:9000;
            fastcgi_index index.php;
            include fastcgi_params;
            fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        }
    }

{{/TOOLS_ADMINER_ENABLE}}
{{#TOOLS_PHPPGADMIN_ENABLE}}
    # PHPPgAdmin
    server {
        listen 80;
        server_name phppgadmin.*;
        
        root /var/www/html/phppgadmin;
        index index.php;
        
        location / {
            try_files $uri $uri/ /index.php?$query_string;
        }
        
        location ~ \.php$ {
            fastcgi_pass 127.0.0.1:9000;
            fastcgi_index index.php;
            include fastcgi_params;
            fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        }
    }

{{/TOOLS_PHPPGADMIN_ENABLE}}
{{#TOOLS_PHPMEMCACHEDADMIN_ENABLE}}
    # PHPMemcachedAdmin
    server {
        listen 80;
        server_name phpmemcachedadmin.*;
        
        root /var/www/html/phpmemcachedadmin;
        index index.php;
        
        location / {
            try_files $uri $uri/ /index.php?$query_string;
        }
        
        location ~ \.php$ {
            fastcgi_pass 127.0.0.1:9000;
            fastcgi_index index.php;
            include fastcgi_params;
            fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        }
    }

{{/TOOLS_PHPMEMCACHEDADMIN_ENABLE}}
{{#TOOLS_PHPMONGO_ENABLE}}
    # PHPMongo
    server {
        listen 80;
        server_name phpmongo.*;
        
        root /var/www/html/phpmongo;
        index index.php;
        
        location / {
            try_files $uri $uri/ /index.php?$query_string;
        }
        
        location ~ \.php$ {
            fastcgi_pass 127.0.0.1:9000;
            fastcgi_index index.php;
            include fastcgi_params;
            fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        }
    }

{{/TOOLS_PHPMONGO_ENABLE}}
{{#TOOLS_OPCACHE_ENABLE}}
    # OPcache
    server {
        listen 80;
        server_name opcache.*;
        
        root /var/www/html/opcache;
        index index.php;
        
        location / {
            try_files $uri $uri/ /index.php?$query_string;
        }
        
        location ~ \.php$ {
            fastcgi_pass 127.0.0.1:9000;
            fastcgi_index index.php;
            include fastcgi_params;
            fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        }
    }

{{/TOOLS_OPCACHE_ENABLE}}
{{#TOOLS_KAFBAT_ENABLE}}
    # Kafbat Kafka UI (Reverse Proxy to Java App)
    server {
        listen 80;
        server_name kafbat.*;
        
        location / {
            proxy_pass http://127.0.0.1:8080;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }

{{/TOOLS_KAFBAT_ENABLE}}
}
