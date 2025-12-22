<!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Project 6 - Stackvo</title>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            margin: 0;
            padding: 20px;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
        }

        .container {
            background: white;
            border-radius: 20px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
            padding: 40px;
            max-width: 800px;
            width: 100%;
        }

        h1 {
            color: #667eea;
            margin: 0 0 10px 0;
            font-size: 2.5em;
        }

        .subtitle {
            color: #666;
            margin: 0 0 30px 0;
            font-size: 1.2em;
        }

        .info-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 20px;
            margin: 30px 0;
        }

        .info-card {
            background: #f8f9fa;
            padding: 20px;
            border-radius: 10px;
            border-left: 4px solid #667eea;
        }

        .info-card h3 {
            margin: 0 0 10px 0;
            color: #333;
            font-size: 0.9em;
            text-transform: uppercase;
            letter-spacing: 1px;
        }

        .info-card p {
            margin: 0;
            color: #667eea;
            font-size: 1.5em;
            font-weight: bold;
        }

        .phpinfo-btn {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 15px 30px;
            border: none;
            border-radius: 10px;
            font-size: 1.1em;
            cursor: pointer;
            text-decoration: none;
            display: inline-block;
            transition: transform 0.2s;
        }

        .phpinfo-btn:hover {
            transform: translateY(-2px);
            box-shadow: 0 5px 15px rgba(102, 126, 234, 0.4);
        }
    </style>
</head>

<body>
    <div class="container">
        <h1>🚀 Project 6</h1>
        <p class="subtitle">Stackvo Multi-Project Environment</p>

        <div class="info-grid">
            <div class="info-card">
                <h3>PHP Version</h3>
                <p><?php echo phpversion(); ?></p>
            </div>
            <div class="info-card">
                <h3>Server</h3>
                <p><?php echo $_SERVER['SERVER_SOFTWARE'] ?? 'Nginx/PHP-FPM'; ?></p>
            </div>
            <div class="info-card">
                <h3>Server Name</h3>
                <p><?php echo $_SERVER['SERVER_NAME']; ?></p>
            </div>
            <div class="info-card">
                <h3>Document Root</h3>
                <p><?php echo $_SERVER['DOCUMENT_ROOT']; ?></p>
            </div>
        </div>

        <div style="text-align: center; margin-top: 30px;">
            <a href="info.php" class="phpinfo-btn">📋 View Full PHP Info</a>
        </div>

        <div style="margin-top: 30px; padding: 20px; background: #e3f2fd; border-radius: 10px;">
            <h3 style="margin-top: 0; color: #1976d2;">✨ Features</h3>
            <ul style="color: #555; line-height: 1.8;">
                <li>Traefik reverse proxy with automatic routing</li>
                <li>MySQL 8.0 database available at localhost:3306</li>
                <li>Redis cache available at localhost:6379</li>
                <li>Docker-based development environment</li>
            </ul>
        </div>
    </div>
</body>

</html>