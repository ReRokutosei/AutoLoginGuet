/// CSS样式
pub const CSS: &str = include_str!("../../assets/style.css");

/// HTML模板
pub const HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Chat</title>
    <style>
        body {
            font-family: "Microsoft YaHei", sans-serif;
            margin: 0;
            padding: 0;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            background: white;
            width: 100%;
        }

        .container {
            width: 100%;
            height: 100%;
            background: white;
            padding: 20px;
            box-sizing: border-box;
            text-align: center;
        }

        h1 {
            color: #333;
            margin-bottom: 20px;
            font-size: 24px;
        }

        .avatar-container {
            display: flex;
            justify-content: center;
            margin-bottom: 20px;
        }

        .avatar {
            width: 100px;
            height: 100px;
            border-radius: 50%;
            object-fit: cover;
            border: 3px solid #f0f0f0;
            box-shadow: 0 5px 15px rgba(0, 0, 0, 0.1);
        }

        .form-group {
            margin-bottom: 15px;
            text-align: left;
        }

        .form-row {
            display: flex;
            align-items: center;
            margin-bottom: 12px;
        }

        .form-row label {
            width: 60px;
            margin-right: 10px;
            font-weight: bold;
            color: #555;
            text-align: right;
            font-size: 14px;
        }

        .form-row input {
            flex: 1;
            padding: 10px;
            border: 2px solid #e1e1e1;
            border-radius: 6px;
            box-sizing: border-box;
            font-size: 14px;
            transition: border-color 0.3s;
        }

        .form-row input:focus {
            border-color: #667eea;
            outline: none;
        }

        label {
            display: block;
            margin-bottom: 6px;
            font-weight: bold;
            color: #555;
            font-size: 14px;
        }

        input[type="text"], input[type="password"], select {
            width: 100%;
            padding: 10px;
            border: 2px solid #e1e1e1;
            border-radius: 6px;
            box-sizing: border-box;
            font-size: 14px;
            transition: border-color 0.3s;
        }

        input[type="text"]:focus, input[type="password"]:focus, select:focus {
            border-color: #667eea;
            outline: none;
        }

        .select-row {
            display: flex;
            justify-content: center;
            margin-bottom: 12px;
        }

        .select-row select {
            width: 180px;
            padding: 10px;
            border: 2px solid #e1e1e1;
            border-radius: 6px;
            box-sizing: border-box;
            font-size: 14px;
            transition: border-color 0.3s;
            text-align: center;
            text-align-last: center;
            -webkit-appearance: none;
            -moz-appearance: none;
            appearance: none;
            background: #fff url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%23666' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3e%3cpolyline points='6,9 12,15 18,9'%3e%3c/polyline%3e%3c/svg%3e") no-repeat right 10px center;
            background-size: 16px;
        }

        .select-row select:focus {
            border-color: #667eea;
            outline: none;
        }

        .checkbox-group {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin: 15px 0;
        }

        .checkbox-item {
            display: flex;
            align-items: center;
        }

        .checkbox-item input[type="checkbox"] {
            transform: scale(1.2);
            margin-right: 6px;
        }

        .checkbox-item label {
            margin-bottom: 0;
            font-size: 14px;
        }

        .button-group {
            display: flex;
            gap: 10px;
            justify-content: space-between;
            margin: 20px 0 15px;
        }

        button {
            flex: 1;
            padding: 12px;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            font-size: 14px;
            font-weight: bold;
            transition: all 0.3s;
        }

        .btn-primary {
            background-color: #667eea;
            color: white;
        }

        .btn-primary:hover {
            background-color: #5a6fd8;
            transform: translateY(-2px);
        }

        .btn-success {
            background-color: #28a745;
            color: white;
        }

        .btn-success:hover {
            background-color: #218838;
            transform: translateY(-2px);
        }

        .btn-info {
            background-color: #17a2b8;
            color: white;
        }

        .btn-info:hover {
            background-color: #138496;
            transform: translateY(-2px);
        }

        .alert {
            padding: 12px;
            border-radius: 6px;
            margin-bottom: 15px;
            text-align: center;
            font-size: 14px;
            min-width: 200px;
            width: 100%;
            box-sizing: border-box;
        }

        .alert-info {
            background-color: #d1ecf1;
            color: #0c5460;
            border: 1px solid #bee5eb;
        }

        .session-logs {
            font-family: "Consolas", "Courier New", monospace;
            font-size: 11px;
            resize: vertical;
            width: 100%;
            min-height: 100px;
            border-radius: 6px;
            border: 1px solid #e1e1e1;
            padding: 8px;
            box-sizing: border-box;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Chat</h1>
        <div class="avatar-container">
            <img src="https://via.placeholder.com/100" alt="Avatar" class="avatar">
        </div>
        <form id="chat-form">
            <div class="form-group">
                <div class="form-row">
                    <label for="username">Username:</label>
                    <input type="text" id="username" name="username" required>
                </div>
                <div class="form-row">
                    <label for="password">Password:</label>
                    <input type="password" id="password" name="password" required>
                </div>
                <div class="form-row">
                    <label for="role">Role:</label>
                    <select id="role" name="role" required>
                        <option value="user">User</option>
                        <option value="admin">Admin</option>
                    </select>
                </div>
            </div>
            <div class="checkbox-group">
                <div class="checkbox-item">
                    <input type="checkbox" id="remember" name="remember">
                    <label for="remember">Remember me</label>
                </div>
                <div class="checkbox-item">
                    <input type="checkbox" id="terms" name="terms" required>
                    <label for="terms">Accept terms</label>
                </div>
            </div>
            <div class="button-group">
                <button type="submit" class="btn-primary">Login</button>
                <button type="button" class="btn-success">Register</button>
                <button type="button" class="btn-info">Forgot Password</button>
            </div>
        </form>
        <div id="session-logs" class="session-logs"></div>
    </div>
    <script>
        document.getElementById('chat-form').addEventListener('submit', function(event) {
            event.preventDefault();
            const formData = new FormData(this);
            const data = Object.fromEntries(formData.entries());
            console.log(data);
            document.getElementById('session-logs').innerText += JSON.stringify(data) + '\n';
        });
    </script>
</body>
</html>
"#;