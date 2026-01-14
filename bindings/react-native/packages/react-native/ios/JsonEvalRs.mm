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

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(createFromMsgpack:(NSArray *)schemaMsgpack
                                       context:(NSString *)context
                                       data:(NSString *)data)
{
    std::string contextStr = [self stdStringFromNSString:context];
    std::string dataStr = [self stdStringFromNSString:data];
    
    // Convert NSArray to std::vector<uint8_t>
    std::vector<uint8_t> msgpackBytes;
    msgpackBytes.reserve([schemaMsgpack count]);
    for (NSNumber *num in schemaMsgpack) {
        msgpackBytes.push_back([num unsignedCharValue]);
    }
    
    std::string handle = JsonEvalBridge::createFromMsgpack(msgpackBytes, contextStr, dataStr);
    return [self stringFromStdString:handle];
}

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(createFromCache:(NSString *)cacheKey
                                       context:(NSString *)context
                                       data:(NSString *)data)
{
    std::string cacheKeyStr = [self stdStringFromNSString:cacheKey];
    std::string contextStr = [self stdStringFromNSString:context];
    std::string dataStr = [self stdStringFromNSString:data];
    
    std::string handle = JsonEvalBridge::createFromCache(cacheKeyStr, contextStr, dataStr);
    return [self stringFromStdString:handle];
}

RCT_EXPORT_METHOD(evaluate:(NSString *)handle
                  data:(NSString *)data
                  context:(NSString *)context
                  pathsJson:(NSString *)pathsJson
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    // Convert Objective-C strings to C++ (required by bridge architecture)
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string dataStr = [self stdStringFromNSString:data];
    std::string contextStr = [self stdStringFromNSString:context];
    std::string pathsJsonStr = [self stdStringFromNSString:pathsJson];
    
    JsonEvalBridge::evaluateAsync(handleStr, dataStr, contextStr, pathsJsonStr,
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

RCT_EXPORT_METHOD(compileLogic:(NSString *)handle
                  logicStr:(NSString *)logicStr
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    try {
        std::string handleStr = [self stdStringFromNSString:handle];
        std::string logicStrCpp = [self stdStringFromNSString:logicStr];
        uint64_t logicId = JsonEvalBridge::compileLogic(handleStr, logicStrCpp);
        if (logicId == 0) {
            reject(@"COMPILE_LOGIC_ERROR", @"Failed to compile logic", nil);
        } else {
            resolve(@(logicId));
        }
    } catch (const std::exception& e) {
        reject(@"COMPILE_LOGIC_ERROR", [NSString stringWithUTF8String:e.what()], nil);
    }
}

RCT_EXPORT_METHOD(runLogic:(NSString *)handle
                  logicId:(nonnull NSNumber *)logicId
                  data:(NSString *)data
                  context:(NSString *)context
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    uint64_t logicIdValue = [logicId unsignedLongLongValue];
    std::string dataStr = [self stdStringFromNSString:data];
    std::string contextStr = [self stdStringFromNSString:context];

    JsonEvalBridge::runLogicAsync(handleStr, logicIdValue, dataStr, contextStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"RUN_LOGIC_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(evaluateLogic:(NSString *)logicStr
                  data:(NSString *)data
                  context:(NSString *)context
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string logicString = [self stdStringFromNSString:logicStr];
    std::string dataStr = [self stdStringFromNSString:data];
    std::string contextStr = [self stdStringFromNSString:context];

    JsonEvalBridge::evaluateLogicAsync(logicString, dataStr, contextStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"EVALUATE_LOGIC_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
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
                  changedPathsJson:(NSString *)changedPathsJson
                  data:(NSString *)data
                  context:(NSString *)context
                  reEvaluate:(BOOL)reEvaluate
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string pathsJsonStr = [self stdStringFromNSString:changedPathsJson];
    std::string dataStr = [self stdStringFromNSString:data];
    std::string contextStr = [self stdStringFromNSString:context];
    
    JsonEvalBridge::evaluateDependentsAsync(handleStr, pathsJsonStr, dataStr, contextStr, reEvaluate,
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

RCT_EXPORT_METHOD(getSchemaValueArray:(NSString *)handle
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    
    JsonEvalBridge::getSchemaValueArrayAsync(handleStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_SCHEMA_VALUE_ARRAY_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(getSchemaValueObject:(NSString *)handle
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    
    JsonEvalBridge::getSchemaValueObjectAsync(handleStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_SCHEMA_VALUE_OBJECT_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
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

RCT_EXPORT_METHOD(getEvaluatedSchemaByPaths:(NSString *)handle
                  pathsJson:(NSString *)pathsJson
                  skipLayout:(BOOL)skipLayout
                  format:(NSInteger)format
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string pathsJsonStr = [self stdStringFromNSString:pathsJson];
    
    JsonEvalBridge::getEvaluatedSchemaByPathsAsync(handleStr, pathsJsonStr, skipLayout, static_cast<int>(format),
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_EVALUATED_SCHEMA_BY_PATHS_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(getSchemaByPath:(NSString *)handle
                  path:(NSString *)path
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string pathStr = [self stdStringFromNSString:path];
    
    JsonEvalBridge::getSchemaByPathAsync(handleStr, pathStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_SCHEMA_BY_PATH_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(getSchemaByPaths:(NSString *)handle
                  pathsJson:(NSString *)pathsJson
                  format:(NSInteger)format
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string pathsJsonStr = [self stdStringFromNSString:pathsJson];
    
    JsonEvalBridge::getSchemaByPathsAsync(handleStr, pathsJsonStr, static_cast<int>(format),
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_SCHEMA_BY_PATH_S_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
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

RCT_EXPORT_METHOD(reloadSchemaMsgpack:(NSString *)handle
                  schemaMsgpack:(NSArray *)schemaMsgpack
                  context:(NSString *)context
                  data:(NSString *)data
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string contextStr = [self stdStringFromNSString:context];
    std::string dataStr = [self stdStringFromNSString:data];
    
    // Convert NSArray to std::vector<uint8_t>
    std::vector<uint8_t> msgpackBytes;
    msgpackBytes.reserve([schemaMsgpack count]);
    for (NSNumber *num in schemaMsgpack) {
        msgpackBytes.push_back([num unsignedCharValue]);
    }
    
    JsonEvalBridge::reloadSchemaMsgpackAsync(handleStr, msgpackBytes, contextStr, dataStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"RELOAD_MSGPACK_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(reloadSchemaFromCache:(NSString *)handle
                  cacheKey:(NSString *)cacheKey
                  context:(NSString *)context
                  data:(NSString *)data
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string cacheKeyStr = [self stdStringFromNSString:cacheKey];
    std::string contextStr = [self stdStringFromNSString:context];
    std::string dataStr = [self stdStringFromNSString:data];
    
    JsonEvalBridge::reloadSchemaFromCacheAsync(handleStr, cacheKeyStr, contextStr, dataStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"RELOAD_CACHE_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
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

RCT_EXPORT_METHOD(enableCache:(NSString *)handle
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    
    JsonEvalBridge::enableCacheAsync(handleStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve(nil);
            } else {
                reject(@"ENABLE_CACHE_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(disableCache:(NSString *)handle
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    
    JsonEvalBridge::disableCacheAsync(handleStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve(nil);
            } else {
                reject(@"DISABLE_CACHE_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(isCacheEnabled:(NSString *)handle)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    bool enabled = JsonEvalBridge::isCacheEnabled(handleStr);
    return @(enabled);
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

RCT_EXPORT_METHOD(resolveLayout:(NSString *)handle
                  evaluate:(BOOL)evaluate
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    
    JsonEvalBridge::resolveLayoutAsync(handleStr, evaluate,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"RESOLVE_LAYOUT_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(compileAndRunLogic:(NSString *)handle
                  logicStr:(NSString *)logicStr
                  data:(NSString *)data
                  context:(NSString *)context
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string logicString = [self stdStringFromNSString:logicStr];
    std::string dataStr = [self stdStringFromNSString:data];
    std::string contextStr = [self stdStringFromNSString:context];
    
    JsonEvalBridge::compileAndRunLogicAsync(handleStr, logicString, dataStr, contextStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"COMPILE_AND_RUN_LOGIC_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
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
                  paths:(NSArray *)paths
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string pathStr = [self stdStringFromNSString:subformPath];
    std::string dataStr = [self stdStringFromNSString:data];
    std::string contextStr = [self stdStringFromNSString:context];

    std::string pathsJsonStr = "";
    if (paths != nil) {
        NSString *jsonString = [self arrayToJsonString:paths];
        pathsJsonStr = [self stdStringFromNSString:jsonString];
    }
    
    JsonEvalBridge::evaluateSubformAsync(handleStr, pathStr, dataStr, contextStr, pathsJsonStr,
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
                  reEvaluate:(BOOL)reEvaluate
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string subformPathStr = [self stdStringFromNSString:subformPath];
    std::string changedPathStr = [self stdStringFromNSString:changedPath];
    std::string dataStr = [self stdStringFromNSString:data];
    std::string contextStr = [self stdStringFromNSString:context];
    
    JsonEvalBridge::evaluateDependentsSubformAsync(handleStr, subformPathStr, changedPathStr, dataStr, contextStr, reEvaluate,
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

RCT_EXPORT_METHOD(getSchemaValueArraySubform:(NSString *)handle
                  subformPath:(NSString *)subformPath
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string pathStr = [self stdStringFromNSString:subformPath];
    
    JsonEvalBridge::getSchemaValueArraySubformAsync(handleStr, pathStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_SCHEMA_VALUE_ARRAY_SUBFORM_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(getSchemaValueObjectSubform:(NSString *)handle
                  subformPath:(NSString *)subformPath
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string pathStr = [self stdStringFromNSString:subformPath];
    
    JsonEvalBridge::getSchemaValueObjectSubformAsync(handleStr, pathStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_SCHEMA_VALUE_OBJECT_SUBFORM_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
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

RCT_EXPORT_METHOD(getEvaluatedSchemaByPathsSubform:(NSString *)handle
                  subformPath:(NSString *)subformPath
                  schemaPathsJson:(NSString *)schemaPathsJson
                  skipLayout:(BOOL)skipLayout
                  format:(NSInteger)format
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string subformPathStr = [self stdStringFromNSString:subformPath];
    std::string schemaPathsJsonStr = [self stdStringFromNSString:schemaPathsJson];
    
    JsonEvalBridge::getEvaluatedSchemaByPathsSubformAsync(handleStr, subformPathStr, schemaPathsJsonStr, skipLayout, static_cast<int>(format),
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_EVALUATED_SCHEMA_BY_PATHS_SUBFORM_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
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

RCT_EXPORT_METHOD(getSchemaByPathSubform:(NSString *)handle
                  subformPath:(NSString *)subformPath
                  schemaPath:(NSString *)schemaPath
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string subformPathStr = [self stdStringFromNSString:subformPath];
    std::string schemaPathStr = [self stdStringFromNSString:schemaPath];
    
    JsonEvalBridge::getSchemaByPathSubformAsync(handleStr, subformPathStr, schemaPathStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_SCHEMA_BY_PATH_SUBFORM_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
            }
        }
    );
}

RCT_EXPORT_METHOD(getSchemaByPathsSubform:(NSString *)handle
                  subformPath:(NSString *)subformPath
                  schemaPathsJson:(NSString *)schemaPathsJson
                  format:(NSInteger)format
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string subformPathStr = [self stdStringFromNSString:subformPath];
    std::string schemaPathsJsonStr = [self stdStringFromNSString:schemaPathsJson];
    
    JsonEvalBridge::getSchemaByPathsSubformAsync(handleStr, subformPathStr, schemaPathsJsonStr, static_cast<int>(format),
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"GET_SCHEMA_BY_PATHS_SUBFORM_ERROR", [NSString stringWithUTF8String:error.c_str()], nil);
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

RCT_EXPORT_METHOD(setTimezoneOffset:(NSString *)handle
                  offsetMinutes:(NSNumber *)offsetMinutes
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    
    // Convert NSNumber to int32_t, use INT32_MIN for null/nil
    int32_t offset = offsetMinutes ? [offsetMinutes intValue] : INT32_MIN;
    
    try {
        JsonEvalBridge::setTimezoneOffset(handleStr, offset);
        resolve(nil);
    } catch (const std::exception& e) {
        reject(@"SET_TIMEZONE_OFFSET_ERROR", [NSString stringWithUTF8String:e.what()], nil);
    }
}

RCT_EXPORT_METHOD(cancel:(NSString *)handle)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    JsonEvalBridge::cancel(handleStr);
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
