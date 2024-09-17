import java.io.*;
import java.net.*;

public class main {
    public static void main(String[] args) {
        int port = 8000;

        try (ServerSocket serverSocket = new ServerSocket(port)){
            System.out.println("Server is listening on port " + port);

            while (true) { 
                try (Socket socket = serverSocket.accept()) {
                    handleRequest(socket);
                } catch (IOException e) {
                    System.err.println("Connection error: " + e.getMessage());
                }
            }
        } catch(IOException e ) {
            System.err.println("Server error :" + e.getMessage());
        }
    }

    private static void handleRequest(Socket socket) throws IOException {
        PrintWriter writer;
        try (BufferedReader reader = new  BufferedReader(new InputStreamReader(socket.getInputStream()))) {
            writer = new PrintWriter(socket.getOutputStream(), true);
            String line = reader.readLine();
            if (line != null && !line.isEmpty()){
                String[] requestParts = line.split(" ");
                String method = requestParts[0];
                String path = requestParts[1];
                
                System.err.println("Received " + method + " request for " + path);
                
                String response =   "HTTP/1.1 200 OK\r\n\r\n"+
                        "Content-type: text/plain\r\n" +
                        "Content-length: 13\r\n\r\n" +
                        "Hello world!";
                writer.println(response);
            }
        }
        writer.close();
    }
}