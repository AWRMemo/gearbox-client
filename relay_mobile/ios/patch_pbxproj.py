#!/usr/bin/env python3
"""Patch Runner.xcodeproj/project.pbxproj to add MlxBridgePackage local SPM dependency.

Usage: python3 patch_pbxproj.py Runner.xcodeproj/project.pbxproj
"""

import sys
import re

PBPATH = sys.argv[1] if len(sys.argv) > 1 else "Runner.xcodeproj/project.pbxproj"

# Deterministic 24-char hex IDs (unique enough for this project)
REF_ID        = "ABCD1234567890ABCDEF012345"  # PBXFileReference
LOCAL_REF_ID  = "ABCD1234567890ABCDEF012346"  # XCLocalSwiftPackageReference
PROD_DEP_ID   = "ABCD1234567890ABCDEF012347"  # XCSwiftPackageProductDependency
BUILD_FILE_ID = "ABCD1234567890ABCDEF012348"  # PBXBuildFile

with open(PBPATH, "r", encoding="utf-8") as f:
    content = f.read()

# 1. Insert PBXFileReference for MlxBridgePackage right after line with FlutterGeneratedPluginSwiftPackage ref
ref_line = (
    f"\t\t{REF_ID} /* MlxBridgePackage */ = {{"
    f"isa = PBXFileReference; lastKnownFileType = wrapper; name = MlxBridgePackage; path = MlxBridgePackage; sourceTree = \"<group>\"; }};\n"
)
content = content.replace(
    "78E0A7A72DC9AD7400C4905E /* FlutterGeneratedPluginSwiftPackage */ = {isa = PBXFileReference; lastKnownFileType = wrapper;",
    f"{REF_ID.strip()} /* MlxBridgePackage */ = {{isa = PBXFileReference; lastKnownFileType = wrapper; name = MlxBridgePackage; path = MlxBridgePackage; sourceTree = \"<group>\"; }};\n\t\t78E0A7A72DC9AD7400C4905E /* FlutterGeneratedPluginSwiftPackage */ = {{isa = PBXFileReference; lastKnownFileType = wrapper;"
)

# 2. Append MlxBridgeProductDependency entry inside XCSwiftPackageProductDependency section
prod_dep = (
    f"\t\t{PROD_DEP_ID} /* MlxBridge */ = {{"
    f"isa = XCSwiftPackageProductDependency; package = {LOCAL_REF_ID} /* XCLocalSwiftPackageReference \"MlxBridgePackage\" */; "
    f"productName = MlxBridge; }};\n"
)
content = content.replace(
    "/* End XCSwiftPackageProductDependency section */",
    prod_dep + "/* End XCSwiftPackageProductDependency section */"
)

# 3. Append PBXBuildFile entry inside PBXBuildFile section
build_file = (
    f"\t\t{BUILD_FILE_ID} /* MlxBridge in Frameworks */ = {{"
    f"isa = PBXBuildFile; productRef = {PROD_DEP_ID} /* MlxBridge */; }};\n"
)
content = content.replace(
    "/* End PBXBuildFile section */",
    build_file + "/* End PBXBuildFile section */"
)

# 4. Add build file to Runner Frameworks build phase
content = content.replace(
    "78A318202AECB46A00862997 /* FlutterGeneratedPluginSwiftPackage in Frameworks */",
    f"78A318202AECB46A00862997 /* FlutterGeneratedPluginSwiftPackage in Frameworks */,\n\t\t\t\t{BUILD_FILE_ID} /* MlxBridge in Frameworks */"
)

# 5. Add product dependency to Runner target packageProductDependencies
content = content.replace(
    "packageProductDependencies = (\n\t\t\t\t78A3181F2AECB46A00862997",
    f"packageProductDependencies = (\n\t\t\t\t{PROD_DEP_ID} /* MlxBridge */,\n\t\t\t\t78A3181F2AECB46A00862997"
)

# 6. Add package reference in PBXProject packageReferences
content = content.replace(
    "packageReferences = (\n\t\t\t\t781AD8BC2B33823900A9FFBB",
    f"packageReferences = (\n\t\t\t\t{LOCAL_REF_ID} /* XCLocalSwiftPackageReference \"MlxBridgePackage\" */,\n\t\t\t\t781AD8BC2B33823900A9FFBB"
)

# 7. Append XCLocalSwiftPackageReference entry inside existing XCLocalSwiftPackageReference section
local_ref = f"\t\t{LOCAL_REF_ID} /* XCLocalSwiftPackageReference \"MlxBridgePackage\" */ = {{isa = XCLocalSwiftPackageReference; relativePath = MlxBridgePackage; }};\n"
content = content.replace(
    "/* End XCLocalSwiftPackageReference section */",
    local_ref + "/* End XCLocalSwiftPackageReference section */"
)

with open(PBPATH, "w", encoding="utf-8") as f:
    f.write(content)

print("Patched project.pbxproj successfully.")
