#import "JsonEvalRs.h"
#import <React/RCTLog.h>
#include "json-eval-bridge.h"

using namespace jsoneval;

@implementation JsonEvalRs

RCT_EXPORT_MODULE()

// Helper methods
- (NSString *)stringFromStdString:(const std::string &)str {
    return [NSString stringWithUTF8String:str.c_str()];
}

- (std::string)stdStringFromNSString:(NSString *)str {
    if (str == nil) return "";
    return std::string([str UTF8String]);
}

- (NSString *)arrayToJsonString:(NSArray *)array {
    NSMutableArray *quotedItems = [NSMutableArray array];
    for (NSString *item in array) {
        [quotedItems addObject:[NSString stringWithFormat:@"\"%@\"", item]];
    }
    return [NSString stringWithFormat:@"[%@]", [quotedItems componentsJoinedByString:@","]];
}

RCT_EXPORT_METHOD(create:(NSString *)schema
                  context:(NSString *)context
                  data:(NSString *)data
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    @try {
        std::string schemaStr = [self stdStringFromNSString:schema];
        std::string contextStr = [self stdStringFromNSString:context];
        std::string dataStr = [self stdStringFromNSString:data];
        
        std::string handle = JsonEvalBridge::create(schemaStr, contextStr, dataStr);
        resolve([self stringFromStdString:handle]);
    } @catch (NSException *exception) {
        reject(@"CREATE_ERROR", exception.reason, nil);
    }
}

RCT_EXPORT_METHOD(evaluate:(NSString *)handle
                  data:(NSString *)data
                  context:(NSString *)context
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string dataStr = [self stdStringFromNSString:data];
    std::string contextStr = [self stdStringFromNSString:context];
    
    JsonEvalBridge::evaluateAsync(handleStr, dataStr, contextStr,
        [resolve, reject](const std::string& result, const std::string& error) {
            if (error.empty()) {
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
                  changedPaths:(NSArray *)changedPaths
                  data:(NSString *)data
                  context:(NSString *)context
                  nested:(BOOL)nested
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    std::string handleStr = [self stdStringFromNSString:handle];
    std::string pathsJson = [self stdStringFromNSString:[self arrayToJsonString:changedPaths]];
    std::string dataStr = [self stdStringFromNSString:data];
    std::string contextStr = [self stdStringFromNSString:context];
    
    JsonEvalBridge::evaluateDependentsAsync(handleStr, pathsJson, dataStr, contextStr, nested,
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

RCT_EXPORT_METHOD(dispose:(NSString *)handle
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)
{
    @try {
        std::string handleStr = [self stdStringFromNSString:handle];
        JsonEvalBridge::dispose(handleStr);
        resolve(nil);
    } @catch (NSException *exception) {
        reject(@"DISPOSE_ERROR", exception.reason, nil);
    }
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
