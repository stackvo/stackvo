<?php
/**
 * denemedir - Welcome Page
 */
?>
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Welcome to denemedir</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
        h1 { color: #333; }
        .info { background: #f4f4f4; padding: 15px; border-radius: 5px; }
    </style>
</head>
<body>
    <h1>🎉 Welcome to denemedir!</h1>
    <div class="info">
        <p><strong>Project:</strong> denemedir</p>
        <p><strong>Domain:</strong> denemedir.loc</p>
        <p><strong>PHP Version:</strong> <?php echo phpversion(); ?></p>
        <p><strong>Document Root:</strong> public</p>
    </div>
    <p>Your project is ready! Start building something amazing.</p>
</body>
</html>
