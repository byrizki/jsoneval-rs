#import "JsonEvalRs.h"
#import <React/RCTLog.h>
#include "json-eval-bridge.h"

using namespace jsoneval;

@implementation JsonEvalRs

RCT_EXPORT_MODULE()

// Helper methods
// Note: These conversions are required by React Native bridge architecture
// We minimize copies where possible, but some are unavoidable due to lifetime management
- (NSString *)stringFromStdString:(const std::string &)str {
    // Creates NSString from UTF-8 C string (copies data)
    return [NSString stringWithUTF8String:str.c_str()];
}

- (std::string)stdStringFromNSString:(NSString *)str {
    if (str == nil) return "";
    // UTF8String returns a pointer to UTF-8 representation (minimal copy)
    // We must copy to std::string for safe lifetime management
    const char* utf8 = [str UTF8String];
    return utf8 ? std::string(utf8) : std::string();
}

- (NSString *)arrayToJsonString:(NSArray *)array {
    NSMutableArray *quotedItems = [NSMutableArray array];
    for (NSString *item in array) {
        [quotedItems addObject:[NSString stringWithFormat:@"\"%@\"", item]];
    }
    return [NSString stringWithFormat:@"[%@]", [quotedItems componentsJoinedByString:@","]];
}

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(create:(NSString *)schema
                                       context:(NSString *)context
                                       data:(NSString *)data)
{
    std::string schemaStr = [self stdStringFromNSString:schema];
    std::string contextStr = [self stdStringFromNSString:context];
    std::string dataStr = [self stdStringFromNSString:data];
    
    std::string handle = JsonEvalBridge::create(schemaStr, contextStr, dataStr);
    return [self stringFromStdString:handle];
}

RCT_EXPORT_METHOD(evaluate:(NSString *)handle
                  data:(NSString *)data
                  context:(NSString *)context
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    // Convert Objective-C strings to C++ (required by bridge architecture)
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string dataStr = [self stdStringFromNSString:data];
    std::string contextStr = [self stdStringFromNSString:context];
    
    JsonEvalBridge::evaluateAsync(handleStr, dataStr, contextStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                // Zero-copy within native: direct pointer access to result
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"EVALUATE_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(validate:(NSString *)handle
                  data:(NSString *)data
                  context:(NSString *)context
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string dataStr = [self stdStringFromNSString:data];
    std::string contextStr = [self stdStringFromNSString:context];
    
    JsonEvalBridge::validateAsync(handleStr, dataStr, contextStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"VALIDATE_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(evaluateDependents:(NSString *)handle
                  changedPath:(NSString *)changedPath
                  data:(NSString *)data
                  context:(NSString *)context
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string pathStr = [self stdStringFromNSString:changedPath];
    std::string dataStr = [self stdStringFromNSString:data];
    std::string contextStr = [self stdStringFromNSString:context];
    
    JsonEvalBridge::evaluateDependentsAsync(handleStr, pathStr, dataStr, contextStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"EVALUATE_DEPENDENTS_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(getEvaluatedSchema:(NSString *)handle
                  skipLayout:(BOOL)skipLayout
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    
    JsonEvalBridge::getEvaluatedSchemaAsync(handleStr, skipLayout,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_SCHEMA_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(getSchemaValue:(NSString *)handle
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    
    JsonEvalBridge::getSchemaValueAsync(handleStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_VALUE_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(getEvaluatedSchemaWithoutParams:(NSString *)handle
                  skipLayout:(BOOL)skipLayout
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    
    JsonEvalBridge::getEvaluatedSchemaWithoutParamsAsync(handleStr, skipLayout,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_SCHEMA_WITHOUT_PARAMS_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(getEvaluatedSchemaByPath:(NSString *)handle
                  path:(NSString *)path
                  skipLayout:(BOOL)skipLayout
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string pathStr = [self stdStringFromNSString:path];
    
    JsonEvalBridge::getEvaluatedSchemaByPathAsync(handleStr, pathStr, skipLayout,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_EVALUATED_SCHEMA_BY_PATH_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(reloadSchema:(NSString *)handle
                  schema:(NSString *)schema
                  context:(NSString *)context
                  data:(NSString *)data
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string schemaStr = [self stdStringFromNSString:schema];
    std::string contextStr = [self stdStringFromNSString:context];
    std::string dataStr = [self stdStringFromNSString:data];
    
    JsonEvalBridge::reloadSchemaAsync(handleStr, schemaStr, contextStr, dataStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"RELOAD_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(cacheStats:(NSString *)handle
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    
    JsonEvalBridge::cacheStatsAsync(handleStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"CACHE_STATS_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(clearCache:(NSString *)handle
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    
    JsonEvalBridge::clearCacheAsync(handleStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"CLEAR_CACHE_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(cacheLen:(NSString *)handle
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    
    JsonEvalBridge::cacheLenAsync(handleStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                NSInteger len = [[NSString stringWithUTF8String:result.c_str()] integerValue];
                resolve(@(len));
            } else {
                reject(@"CACHE_LEN_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(validatePaths:(NSString *)handle
                  data:(NSString *)data
                  context:(NSString *)context
                  paths:(NSArray *)paths
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string dataStr = [self stdStringFromNSString:data];
    std::string contextStr = [self stdStringFromNSString:context];
    std::string pathsJson = paths ? [self stdStringFromNSString:[self arrayToJsonString:paths]] : "";
    
    JsonEvalBridge::validatePathsAsync(handleStr, dataStr, contextStr, pathsJson,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"VALIDATE_PATHS_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

// ============================================================================
// Subform Methods
// ============================================================================

RCT_EXPORT_METHOD(evaluateSubform:(NSString *)handle
                  subformPath:(NSString *)subformPath
                  data:(NSString *)data
                  context:(NSString *)context
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string pathStr = [self stdStringFromNSString:subformPath];
    std::string dataStr = [self stdStringFromNSString:data];
    std::string contextStr = [self stdStringFromNSString:context];
    
    JsonEvalBridge::evaluateSubformAsync(handleStr, pathStr, dataStr, contextStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"EVALUATE_SUBFORM_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(validateSubform:(NSString *)handle
                  subformPath:(NSString *)subformPath
                  data:(NSString *)data
                  context:(NSString *)context
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string pathStr = [self stdStringFromNSString:subformPath];
    std::string dataStr = [self stdStringFromNSString:data];
    std::string contextStr = [self stdStringFromNSString:context];
    
    JsonEvalBridge::validateSubformAsync(handleStr, pathStr, dataStr, contextStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"VALIDATE_SUBFORM_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(evaluateDependentsSubform:(NSString *)handle
                  subformPath:(NSString *)subformPath
                  changedPath:(NSString *)changedPath
                  data:(NSString *)data
                  context:(NSString *)context
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string subformPathStr = [self stdStringFromNSString:subformPath];
    std::string changedPathStr = [self stdStringFromNSString:changedPath];
    std::string dataStr = [self stdStringFromNSString:data];
    std::string contextStr = [self stdStringFromNSString:context];
    
    JsonEvalBridge::evaluateDependentsSubformAsync(handleStr, subformPathStr, changedPathStr, dataStr, contextStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"EVALUATE_DEPENDENTS_SUBFORM_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(resolveLayoutSubform:(NSString *)handle
                  subformPath:(NSString *)subformPath
                  evaluate:(BOOL)evaluate
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string pathStr = [self stdStringFromNSString:subformPath];
    
    JsonEvalBridge::resolveLayoutSubformAsync(handleStr, pathStr, evaluate,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"RESOLVE_LAYOUT_SUBFORM_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(getEvaluatedSchemaSubform:(NSString *)handle
                  subformPath:(NSString *)subformPath
                  resolveLayout:(BOOL)resolveLayout
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string pathStr = [self stdStringFromNSString:subformPath];
    
    JsonEvalBridge::getEvaluatedSchemaSubformAsync(handleStr, pathStr, resolveLayout,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_EVALUATED_SCHEMA_SUBFORM_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(getSchemaValueSubform:(NSString *)handle
                  subformPath:(NSString *)subformPath
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string pathStr = [self stdStringFromNSString:subformPath];
    
    JsonEvalBridge::getSchemaValueSubformAsync(handleStr, pathStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_SCHEMA_VALUE_SUBFORM_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(getEvaluatedSchemaWithoutParamsSubform:(NSString *)handle
                  subformPath:(NSString *)subformPath
                  resolveLayout:(BOOL)resolveLayout
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string pathStr = [self stdStringFromNSString:subformPath];
    
    JsonEvalBridge::getEvaluatedSchemaWithoutParamsSubformAsync(handleStr, pathStr, resolveLayout,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_EVALUATED_SCHEMA_WITHOUT_PARAMS_SUBFORM_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(getEvaluatedSchemaByPathSubform:(NSString *)handle
                  subformPath:(NSString *)subformPath
                  schemaPath:(NSString *)schemaPath
                  skipLayout:(BOOL)skipLayout
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string subformPathStr = [self stdStringFromNSString:subformPath];
    std::string schemaPathStr = [self stdStringFromNSString:schemaPath];
    
    JsonEvalBridge::getEvaluatedSchemaByPathSubformAsync(handleStr, subformPathStr, schemaPathStr, skipLayout,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_EVALUATED_SCHEMA_BY_PATH_SUBFORM_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(getSubformPaths:(NSString *)handle
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    
    JsonEvalBridge::getSubformPathsAsync(handleStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_SUBFORM_PATHS_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(hasSubform:(NSString *)handle
                  subformPath:(NSString *)subformPath
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string pathStr = [self stdStringFromNSString:subformPath];
    
    JsonEvalBridge::hasSubformAsync(handleStr, pathStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"HAS_SUBFORM_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(dispose:(NSString *)handle)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    JsonEvalBridge::dispose(handleStr);
    return nil;
}

RCT_EXPORT_METHOD(version:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    @try {
        std::string ver = JsonEvalBridge::version();
        resolve([self stringFromStdString:ver]);
    } @catch (NSException *exception) {
        reject(@"VERSION_ERROR", exception.reason, nil);
    }
}

RCT_EXPORT_METHOD(multiply:(double)a
                  b:(double)b
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    resolve(@(a * b));
}

@end
