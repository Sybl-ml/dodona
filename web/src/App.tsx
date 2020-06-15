import React, {Component} from 'react';
import './App.css';


class App extends Component {

  state = {
    contacts: [],
    apiMessage: ""
  }

  componentDidMount() {
    fetch("http://localhost:3001/api/hello")
    .then(res => res.text())
    .then((data) => {
      this.setState({apiMessage: data})
    })
    .catch(console.log)
  }

  public render() {
    return (
      <div className="App">
      <header className="App-header">
        <img src="logo.png" className="App-logo" alt="logo" />
        <h1>
          Welcome to Sybl
        </h1>
        <h3>Distributed ML with Ensemble Methods</h3>
        <p>
          {this.state.apiMessage}
        </p>
      </header>
    </div>
    );
  }
}

export default App;