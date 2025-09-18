# 配置参数
$config = @{
    Username = "xxxx"
    Password = "xxxx"
    BaseUrl = "https://nicdrcom.guet.edu.cn/Self"
}

$session = New-Object Microsoft.PowerShell.Commands.WebRequestSession
$urls = @{
    Login = "$($config.BaseUrl)/login/"
    Verify = "$($config.BaseUrl)/login/verify"
    Captcha = "$($config.BaseUrl)/login/randomCode"
    Dashboard = "$($config.BaseUrl)/dashboard"
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
    try {
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
        $hash = $md5.ComputeHash($bytes)
        return -join ($hash | ForEach-Object { $_.ToString("x2") })
    } finally {
        $md5.Dispose()
    }
}

function Invoke-SafeWebRequest {
    param($Uri, $Method = "GET", $Body = $null, $ContentType = $null)
    
    try {
        $params = @{
            Uri = $Uri
            WebSession = $session
            Headers = $headers
            Method = $Method
            ErrorAction = "Stop"
        }
        
        if ($Body) { $params.Body = $Body }
        if ($ContentType) { $params.ContentType = $ContentType }
        
        return Invoke-WebRequest @params
    } catch {
        return $null
    }
}

function Get-LoginPage {
    return (Invoke-SafeWebRequest -Uri $urls.Login)?.Content
}

function Get-CheckCode($HtmlContent) {
    if ($HtmlContent -match 'name="checkcode" value="([^"]*)"') {
        return $matches[1]
    }
    return $null
}

function Get-CaptchaImage {
    $timestamp = Get-Date -UFormat %s
    $captchaUrl = "$($urls.Captcha)?t=$timestamp"
    return [bool](Invoke-SafeWebRequest -Uri $captchaUrl)
}

function Invoke-SiteLogin($Username, $Password, $Checkcode) {
    $randomCode = "{0:D4}" -f (Get-Random -Minimum 0 -Maximum 9999)
    $encryptedPassword = Get-MD5Hash -Text $Password
    
    $loginData = @{
        account = $Username
        password = $encryptedPassword
        checkcode = $Checkcode
        code = $randomCode
    }
    
    if (-not (Invoke-SafeWebRequest -Uri $urls.Verify -Method "POST" -Body $loginData)) {
        return $false, $null
    }
    
    $dashboardResponse = Invoke-SafeWebRequest -Uri $urls.Dashboard
    if (-not $dashboardResponse) { return $false, $null }
    
    $dashboardContent = $dashboardResponse.Content
    $loginSuccess = $dashboardContent -match "leftFlow"
    
    return $loginSuccess, $dashboardContent
}

function Get-RemainingFlow($DashboardHtml) {
    $patterns = @(
        '<dt>\s*(\d+(?:\.\d+)?)\s*<small[^>]*>M</small>\s*</dt>\s*<dd>剩余流量</dd>',
        '"leftFlow":(\d+\.?\d*)'
    )
    
    foreach ($pattern in $patterns) {
        if ($DashboardHtml -match $pattern) {
            return [double]$matches[1]
        }
    }
    
    return $null
}

function Format-FlowInfo($LeftFlow) {
    switch ($LeftFlow) {
        0.0 { return "校园网 流量耗尽，限速不限量生效" }
        { $_ -lt 1024.0 } { return "校园网 剩余流量 {0:F2} MB" -f $LeftFlow }
        default { return "校园网 剩余流量 {0:F2} GB" -f ($LeftFlow / 1024.0) }
    }
}

function Show-Notification($Message) {
    try {
        Import-Module BurntToast -ErrorAction SilentlyContinue
        New-BurntToastNotification -Text $Message -AppLogo None -ErrorAction SilentlyContinue
    } catch {
        # 静默失败
    }
}

$loginPageContent = Get-LoginPage
if (-not $loginPageContent) { exit 1 }

$checkcode = Get-CheckCode -HtmlContent $loginPageContent
if (-not $checkcode) { exit 1 }

if (-not (Get-CaptchaImage)) { exit 1 }

$loginSuccess, $dashboardContent = Invoke-SiteLogin -Username $config.Username -Password $config.Password -Checkcode $checkcode
if (-not $loginSuccess) { exit 1 }

$leftFlow = Get-RemainingFlow -DashboardHtml $dashboardContent
if ($null -eq $leftFlow) { exit 1 }

$flowInfo = Format-FlowInfo -LeftFlow $leftFlow
Show-Notification -Message $flowInfo
