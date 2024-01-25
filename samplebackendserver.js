const myNeonModule = require('.');

myNeonModule.writeToFile('./out.txt', 'Hello, Neon!', (err) => {
    if (err) {
        console.error('Failed to write to file:', err);
    } else {
        console.log('File written successfully');
    }
});
