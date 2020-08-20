import React from "react"
import { Navbar, Nav, NavDropdown, Button } from 'react-bootstrap';

import MemoLogo from '../icons/Logo.js';


const Header= () => {

        return (
		<Navbar fixed="top">
			<Navbar.Brand href="#home">
			
				<MemoLogo 
					height="30"
					className="d-inline-block align-top"
				/> 
			</Navbar.Brand>
				
			<Nav>
				<NavDropdown title="Product" id="basic-nav-dropdown">
					<div class="square"></div>
					<NavDropdown.Item href="#action/3.1">Action</NavDropdown.Item>
					<NavDropdown.Item href="#action/3.2">Another action</NavDropdown.Item>
					<NavDropdown.Item href="#action/3.3">Something</NavDropdown.Item>
					<NavDropdown.Divider />
					<NavDropdown.Item href="#action/3.4">Separated link</NavDropdown.Item>
				</NavDropdown>
				<NavDropdown title="Resources" id="basic-nav-dropdown">
					<div class="square"></div>
					<NavDropdown.Item href="#action/3.1">Action</NavDropdown.Item>
					<NavDropdown.Item href="#action/3.2">Another action</NavDropdown.Item>
					<NavDropdown.Item href="#action/3.3">Something</NavDropdown.Item>
					<NavDropdown.Divider />
					<NavDropdown.Item href="#action/3.4">Separated link</NavDropdown.Item>
				</NavDropdown>

				<Nav.Link href="#pricing">Pricing</Nav.Link>
    		</Nav>
    		<Navbar.Collapse className="justify-content-end">
				<Nav>
					<Nav.Link href="#Login">Sign In</Nav.Link>
				</Nav>
				<Button variant="primary" href="#SignUp"><b>SIGN UP NOW</b></Button>
			</Navbar.Collapse>
  		</Navbar>
        );
    };
export default Header;