import type { RestfulApi } from "../types/restfulApi";

/**
 * 检查是否有某个接口的权限
 * @param method HTTP 方法（GET / POST / PUT / DELETE）
 * @param path   接口路径
 * @returns boolean
 */
export function hasPermission(method: string, path: string): boolean {
  try {
    const restful_apis: RestfulApi[] = JSON.parse(
      localStorage.getItem("restful_apis") || "[]"
    );

    return restful_apis.some(
      (api) =>
        api.method.toUpperCase() === method.toUpperCase() &&
        api.path === path
    );
  } catch {
    return false;
  }
}