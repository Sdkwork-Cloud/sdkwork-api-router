import {
  assertUserCenterValidationPreflightCompatibility,
  USER_CENTER_VALIDATION_SOURCE_PACKAGE_NAME,
  createUserCenterValidationInteropContract,
  createUserCenterValidationPluginDefinition,
  createUserCenterValidationPreflightReport,
  createUserCenterValidationSnapshot,
  requireUserCenterProtectedToken,
  resolveUserCenterProtectedToken,
  type UserCenterProtectedTokenRequirementOptions,
  type UserCenterProtectedTokenResolutionOptions,
  type UserCenterValidationInteropContract,
  type UserCenterValidationPluginDefinition,
  type UserCenterValidationPreflightReport,
  type UserCenterValidationSnapshot,
} from "../../../../../../sdkwork-appbase/packages/pc-react/identity/sdkwork-user-center-validation-pc-react/src/index.ts";
import {
  createRouterPortalUserCenterConfig,
  createRouterPortalUserCenterPluginDefinition,
  type CreateRouterPortalUserCenterConfigOptions,
  type CreateRouterPortalUserCenterPluginDefinitionOptions,
} from './userCenter';

export type RouterPortalUserCenterValidationSnapshot = UserCenterValidationSnapshot;
export type RouterPortalUserCenterValidationPluginDefinition = UserCenterValidationPluginDefinition;
export type RouterPortalUserCenterValidationInteropContract = UserCenterValidationInteropContract;
export type RouterPortalUserCenterValidationPreflightReport = UserCenterValidationPreflightReport;
export type RouterPortalProtectedTokenResolutionOptions = UserCenterProtectedTokenResolutionOptions;
export type RouterPortalProtectedTokenRequirementOptions = UserCenterProtectedTokenRequirementOptions;

export interface CreateRouterPortalUserCenterValidationPreflightOptions
  extends CreateRouterPortalUserCenterConfigOptions {
  peerContract: RouterPortalUserCenterValidationInteropContract;
}

export const ROUTER_PORTAL_USER_CENTER_VALIDATION_SOURCE_PACKAGE =
  USER_CENTER_VALIDATION_SOURCE_PACKAGE_NAME;
export const ROUTER_PORTAL_USER_CENTER_VALIDATION_PLUGIN_PACKAGES = [
  "sdkwork-router-portal-validation",
] as const;

export function createRouterPortalUserCenterValidationSnapshot(
  options: CreateRouterPortalUserCenterConfigOptions = {},
): RouterPortalUserCenterValidationSnapshot {
  return createUserCenterValidationSnapshot(createRouterPortalUserCenterConfig(options));
}

export function createRouterPortalUserCenterValidationInteropContract(
  options: CreateRouterPortalUserCenterConfigOptions = {},
): RouterPortalUserCenterValidationInteropContract {
  return createUserCenterValidationInteropContract(
    createRouterPortalUserCenterValidationSnapshot(options),
  );
}

export function createRouterPortalUserCenterValidationPluginDefinition(
  options: CreateRouterPortalUserCenterPluginDefinitionOptions = {},
): RouterPortalUserCenterValidationPluginDefinition {
  return createUserCenterValidationPluginDefinition({
    ...options,
    packageNames: options.packageNames ?? [...ROUTER_PORTAL_USER_CENTER_VALIDATION_PLUGIN_PACKAGES],
    title: options.title ?? "SDKWORK Router Portal",
    userCenterPlugin: createRouterPortalUserCenterPluginDefinition(options),
  });
}

export function createRouterPortalUserCenterValidationPreflightReport(
  options: CreateRouterPortalUserCenterValidationPreflightOptions,
): RouterPortalUserCenterValidationPreflightReport {
  const { peerContract, ...configOptions } = options;
  return createUserCenterValidationPreflightReport({
    peerContract,
    snapshot: createRouterPortalUserCenterValidationSnapshot(configOptions),
  });
}

export function assertRouterPortalUserCenterValidationPreflight(
  options: CreateRouterPortalUserCenterValidationPreflightOptions,
): RouterPortalUserCenterValidationPreflightReport {
  const { peerContract, ...configOptions } = options;
  return assertUserCenterValidationPreflightCompatibility({
    peerContract,
    snapshot: createRouterPortalUserCenterValidationSnapshot(configOptions),
  });
}

export function resolveRouterPortalProtectedToken(
  options: RouterPortalProtectedTokenResolutionOptions,
): string | null {
  return resolveUserCenterProtectedToken(options);
}

export function requireRouterPortalProtectedToken(
  options: RouterPortalProtectedTokenRequirementOptions,
): string {
  return requireUserCenterProtectedToken(options);
}
