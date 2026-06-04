param(
    [string]$PbPath = "Runner.xcodeproj\project.pbxproj"
)

$REF_ID        = "ABCD1234567890ABCDEF012345"
$LOCAL_REF_ID  = "ABCD1234567890ABCDEF012346"
$PROD_DEP_ID   = "ABCD1234567890ABCDEF012347"
$BUILD_FILE_ID = "ABCD1234567890ABCDEF012348"

$content = Get-Content $PbPath -Raw

# 1. PBXFileReference for MlxBridgePackage
$content = $content -replace '78E0A7A72DC9AD7400C4905E /\* FlutterGeneratedPluginSwiftPackage \*/ = \{isa = PBXFileReference; lastKnownFileType = wrapper;', "`t`t$REF_ID /* MlxBridgePackage */ = {isa = PBXFileReference; lastKnownFileType = wrapper; name = MlxBridgePackage; path = MlxBridgePackage; sourceTree = `"`"<group>`"`"; };`n`t`t78E0A7A72DC9AD7400C4905E /* FlutterGeneratedPluginSwiftPackage */ = {isa = PBXFileReference; lastKnownFileType = wrapper;"

# 2. XCSwiftPackageProductDependency
$content = $content -replace '/\* End XCSwiftPackageProductDependency section \*/', "`t`t$PROD_DEP_ID /* MlxBridge */ = {isa = XCSwiftPackageProductDependency; package = $LOCAL_REF_ID /* XCLocalSwiftPackageReference `"MlxBridgePackage`" */; productName = MlxBridge; };`n/* End XCSwiftPackageProductDependency section */"

# 3. PBXBuildFile
$content = $content -replace '/\* End PBXBuildFile section \*/', "`t`t$BUILD_FILE_ID /* MlxBridge in Frameworks */ = {isa = PBXBuildFile; productRef = $PROD_DEP_ID /* MlxBridge */; };`n/* End PBXBuildFile section */"

# 4. Link in Frameworks
$content = $content -replace '78A318202AECB46A00862997 /\* FlutterGeneratedPluginSwiftPackage in Frameworks \*/', "78A318202AECB46A00862997 /* FlutterGeneratedPluginSwiftPackage in Frameworks */,`n`t`t`t`t$BUILD_FILE_ID /* MlxBridge in Frameworks */"

# 5. Runner product deps
$content = $content -replace 'packageProductDependencies = \(\`n`t`t`t`t78A3181F2AECB46A00862997', "packageProductDependencies = (`n`t`t`t`t$PROD_DEP_ID /* MlxBridge */,`n`t`t`t`t78A3181F2AECB46A00862997"

# 6. Project package references
$content = $content -replace 'packageReferences = \(\`n`t`t`t`t781AD8BC2B33823900A9FFBB', "packageReferences = (`n`t`t`t`t$LOCAL_REF_ID /* XCLocalSwiftPackageReference `"MlxBridgePackage`" */,`n`t`t`t`t781AD8BC2B33823900A9FFBB"

# 7. XCLocalSwiftPackageReference
$content = $content -replace '/\* End XCLocalSwiftPackageReference section \*/', "`t`t$LOCAL_REF_ID /* XCLocalSwiftPackageReference `"MlxBridgePackage`" */ = {isa = XCLocalSwiftPackageReference; relativePath = MlxBridgePackage; };`n/* End XCLocalSwiftPackageReference section */"

Set-Content $PbPath $content -Encoding UTF8
Write-Host "Patched project.pbxproj successfully."
