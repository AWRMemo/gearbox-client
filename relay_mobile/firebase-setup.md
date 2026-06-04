## FCM / Firebase Setup

You need to create a Firebase project and generate credentials. Here's what to do:

### 1. Create Firebase Project
1. Go to https://console.firebase.google.com/
2. Create a new project named "Gearbox Relay" (or use an existing one)
3. Enable **Cloud Messaging** (Build → Cloud Messaging)

### 2. Android App Registration
1. In Firebase Console → Project Overview → Add app → Android
2. Package name: `com.gearbox.relay`
3. Download `google-services.json`
4. Replace the placeholder at `relay_mobile/android/app/google-services.json` with the real file

### 3. Server-side Service Account Key
1. Firebase Console → Project Settings → Service Accounts
2. Click "Generate New Private Key" → download JSON file
3. Set two environment variables on the sync server:
   ```bash
   export FCM_PROJECT_ID="your-project-id"
   export FCM_SERVICE_ACCOUNT_JSON='{paste the entire JSON file contents here}'
   ```

### 4. Environment Variables (local dev)
Add to your sync server `.env` or launch script:
```bash
FCM_PROJECT_ID=relay-gearbox
FCM_SERVICE_ACCOUNT_JSON='{...}'  # single-quoted, no line breaks
```

The server reads these in `relay-sync-server/src/push.rs` and uses them to authenticate with FCM's HTTP v1 API.
