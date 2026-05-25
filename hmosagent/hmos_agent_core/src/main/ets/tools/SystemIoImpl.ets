import { http } from '@kit.NetworkKit';
import { fileIo as fs } from '@kit.CoreFileKit';
import { BusinessError } from '@kit.BasicServicesKit';

/**
 * Host IO implementation for HmosAgent.
 *
 * Provides the concrete HTTP and filesystem operations that Rust's
 * SystemCallbacks delegate to. Each method is standalone and may be
 * called from any thread via the NAPI thread-safe function bridge.
 */
export class SystemIoImpl {

  /**
   * Perform a blocking HTTP POST and return the response body as a string.
   * Called from Rust's post_fn → post_fn_proxy → tsfn → this method.
   */
  public httpPost(url: string, body: string): string {
    try {
      const request = http.createHttp();
      const response = request.requestSync(url, {
        method: http.RequestMethod.POST,
        header: {
          'Content-Type': 'application/json',
        },
        extraData: body,
        connectTimeout: 30000,
        readTimeout: 120000,
      });

      if (request.destroy) {
        request.destroy();
      }

      return typeof response.result === 'string'
        ? response.result
        : JSON.stringify(response.result);
    } catch (err) {
      const e = err as BusinessError;
      return JSON.stringify({
        status: 'error',
        error: `HTTP POST failed: ${e.message}`,
        code: e.code,
      });
    }
  }

  /**
   * Read a file from the sandbox filesDir.
   */
  public readFile(path: string): string {
    try {
      const file = fs.openSync(path, fs.OpenMode.READ_ONLY);
      const stat = fs.statSync(path);
      const buf = new ArrayBuffer(stat.size);
      fs.readSync(file.fd, buf);
      fs.closeSync(file);
      return String.fromCharCode(...new Uint8Array(buf));
    } catch (err) {
      const e = err as BusinessError;
      return JSON.stringify({
        status: 'error',
        error: `Read failed: ${e.message}`,
      });
    }
  }

  /**
   * Write content to a file in the sandbox filesDir.
   */
  public writeFile(path: string, content: string): boolean {
    try {
      const file = fs.openSync(path, fs.OpenMode.CREATE | fs.OpenMode.WRITE_ONLY);
      fs.writeSync(file.fd, content);
      fs.closeSync(file);
      return true;
    } catch (err) {
      console.error(`SystemIoImpl.writeFile failed: ${err}`);
      return false;
    }
  }

  /**
   * Check if a file exists.
   */
  public fileExists(path: string): boolean {
    try {
      return fs.accessSync(path);
    } catch (_e) {
      return false;
    }
  }
}
