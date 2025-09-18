# 配置你的参数
$config = @{
    Username = "xxxx"
    Password = "xxxx"
    ISP = ""  # 运营商: ""(校园网), "@cmcc"(中国移动), "@unicom"(中国联通), "@telecom"(中国电信), "@glgd"(中国广电)
    LoginIP = "http://10.0.1.5"
}

Import-Module BurntToast

function Invoke-Login {
    param($LoginIP, $Username, $ISP, $Password)
    
    $fullUsername = $Username + $ISP
    $body = "callback=dr1003&DDDDD=$fullUsername&upass=$Password&0MKKey=123456"
    $uri = "$LoginIP/drcom/login"
    
    try {
        $response = Invoke-WebRequest -Uri $uri -Method POST -Body $body -ContentType "application/x-www-form-urlencoded" -ErrorAction Stop
        
        switch -Regex ($response.Content) {
            "注销页|认证成功页|Dr\.COMWebLoginID_3\.htm|`"result`":1" { return "SUCCESS" }
            "msga='ldap auth error'|ldap auth error|Msg=01" { return "CONFIG_ERROR" }
            default { return "UNKNOWN_ERROR" }
        }
    } catch {
        return "NETWORK_ERROR"
    }
}

function Get-NetworkUsage {
    param($Username, $Password)
    
    $session = New-Object Microsoft.PowerShell.Commands.WebRequestSession
    $baseUrl = "https://nicdrcom.guet.edu.cn/Self"
    $urls = @{
        Login = "$baseUrl/login/"
        Verify = "$baseUrl/login/verify"
        Captcha = "$baseUrl/login/randomCode"
        Dashboard = "$baseUrl/dashboard"
    }
    
    $headers = @{
        "User-Agent" = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36"
        "Accept" = "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"
        "Accept-Language" = "zh-CN,zh;q=0.9"
        "Connection" = "keep-alive"
        "Upgrade-Insecure-Requests" = "1"
    }

    function Get-MD5Hash($Text) {
        $md5 = [System.Security.Cryptography.MD5]::Create()
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
        $hash = $md5.ComputeHash($bytes)
        return -join ($hash | ForEach-Object { $_.ToString("x2") })
    }

    function Get-LoginPage {
        try {
            return (Invoke-WebRequest -Uri $urls.Login -WebSession $session -Headers $headers -Method GET).Content
        } catch {
            return $null
        }
    }

    function Get-CheckCode($HtmlContent) {
        if ($HtmlContent -match 'name="checkcode" value="([^"]*)"') {
            return $matches[1]
        }
        return $null
    }

    function Get-Captcha {
        $timestamp = Get-Date -UFormat %s
        $captchaUrl = "$($urls.Captcha)?t=$timestamp"
        try {
            Invoke-WebRequest -Uri $captchaUrl -WebSession $session -Method GET | Out-Null
            return $true
        } catch {
            return $false
        }
    }

    function Invoke-SiteLogin($Account, $Password, $Checkcode) {
        $randomCode = "{0:D4}" -f (Get-Random -Minimum 0 -Maximum 9999)
        $encryptedPassword = Get-MD5Hash -Text $Password
        
        $loginData = @{
            account = $Account
            password = $encryptedPassword
            checkcode = $Checkcode
            code = $randomCode
        }
        
        try {
            Invoke-WebRequest -Uri $urls.Verify -WebSession $session -Headers $headers -Method POST -Body $loginData -AllowInsecureRedirect | Out-Null
            
            $dashboardResponse = Invoke-WebRequest -Uri $urls.Dashboard -WebSession $session -Headers $headers -Method GET -AllowInsecureRedirect
            return ($dashboardResponse.Content -match "leftFlow"), $dashboardResponse.Content
        } catch {
            return $false, $null
        }
    }

    function Get-RemainingFlow($DashboardHtml) {
        if ($DashboardHtml -match '<dt>\s*(\d+(?:\.\d+)?)\s*<small[^>]*>M</small>\s*</dt>\s*<dd>剩余流量</dd>') {
            return [double]$matches[1]
        }
        
        if ($DashboardHtml -match '"leftFlow":(\d+\.?\d*)') {
            return [double]$matches[1]
        }
        
        return $null
    }

    function Format-FlowInfo($LeftFlow) {
        switch ($LeftFlow) {
            0.0 { return "流量耗尽，限速不限量生效" }
            { $_ -lt 1024.0 } { return "剩余流量 {0:F2} MB" -f $LeftFlow }
            default { return "剩余流量 {0:F2} GB" -f ($LeftFlow / 1024.0) }
        }
    }

    $loginPageContent = Get-LoginPage
    if (-not $loginPageContent) { return $null }

    $checkcode = Get-CheckCode -HtmlContent $loginPageContent
    if (-not $checkcode) { return $null }

    Get-Captcha | Out-Null

    $loginSuccess, $dashboardContent = Invoke-SiteLogin -Account $Username -Password $Password -Checkcode $checkcode
    if (-not $loginSuccess) { return $null }

    $leftFlow = Get-RemainingFlow -DashboardHtml $dashboardContent
    if ($null -eq $leftFlow) { return $null }

    return [PSCustomObject]@{
        LeftFlowMB = $leftFlow
        LeftFlowGB = if ($leftFlow -ge 1024) { [Math]::Round($leftFlow / 1024.0, 2) } else { $null }
        FormattedFlowInfo = Format-FlowInfo -LeftFlow $leftFlow
    }
}

$loginJob = Start-Job -Name "Login" -ScriptBlock ${function:Invoke-Login} -ArgumentList $config.LoginIP, $config.Username, $config.ISP, $config.Password
$flowJob = Start-Job -Name "FlowQuery" -ScriptBlock ${function:Get-NetworkUsage} -ArgumentList $config.Username, $config.Password

$results = $loginJob, $flowJob | Wait-Job | Receive-Job
$loginResult, $flowResult = $results

Remove-Job -Job $loginJob, $flowJob

$loginMessages = @{
    SUCCESS = "登录成功！"
    CONFIG_ERROR = "登录失败！请检查配置信息"
    UNKNOWN_ERROR = "登录异常！请检查代理或硬件"
    NETWORK_ERROR = "登录失败！网络请求错误"
}

$loginMessage = $loginMessages[$loginResult] ?? "登录状态未知"

$message = if ($loginResult -eq "SUCCESS" -and $flowResult) {
    "$loginMessage$($flowResult.FormattedFlowInfo)"
} else {
    $loginMessage
}

New-BurntToastNotification -Text $message -Applogo None