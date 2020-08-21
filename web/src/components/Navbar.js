import React from "react"
import { Navbar, Nav, NavDropdown,} from 'react-bootstrap';
import styled from "styled-components";
import MemoLogo from '../icons/Logo.js';
import {PrimaryButton} from './Buttons';

const HeaderBar = styled(Navbar)`
	min-height: 4rem;
    background: linear-gradient(${({ theme }) => theme.body}, ${({ theme }) => theme.body} 90%, transparent 100%);
    transition: all 0.25s linear;
`;


const Square = styled.div`
	position: absolute;
	top: -5%;
	left: 45%;
	width: 10%;
	height: 10%;
	filter: drop-shadow(0 1px 2px 0 0 rgba(0,0,0,.5));
	background-color:${({ theme }) => theme.body};
	transform: rotate(45deg);
`;


const Header = ({theme}) => {
        return (
		<HeaderBar sticky="top">
			<HeaderBar.Brand href="#home">
				<MemoLogo 
					theme={theme} 
				/> 
			</HeaderBar.Brand>
				
			<Nav>
				<NavDropdown title="Product" id="basic-nav-dropdown">
					<Square></Square>
					<NavDropdown.Item href="#action/3.1">Action</NavDropdown.Item>
					<NavDropdown.Item href="#action/3.2">Another action</NavDropdown.Item>
					<NavDropdown.Item href="#action/3.3">Something</NavDropdown.Item>
					<NavDropdown.Divider />
					<NavDropdown.Item href="#action/3.4">Separated link</NavDropdown.Item>
				</NavDropdown>
				<NavDropdown title="Resources" id="basic-nav-dropdown">
					<Square></Square>
					<NavDropdown.Item href="#action/3.1">Action</NavDropdown.Item>
					<NavDropdown.Item href="#action/3.2">Another action</NavDropdown.Item>
					<NavDropdown.Item href="#action/3.3">Something</NavDropdown.Item>
					<NavDropdown.Divider />
					<NavDropdown.Item href="#action/3.4">Separated link</NavDropdown.Item>
				</NavDropdown>

				<Nav.Link href="#pricing">Pricing</Nav.Link>
    		</Nav>
    		<HeaderBar.Collapse className="justify-content-end">
				<Nav>
					<Nav.Link href="#Login">Sign In</Nav.Link>
				</Nav>
				<PrimaryButton variant="primary" href="#SignUp">SIGN UP NOW</PrimaryButton>
			</HeaderBar.Collapse>
  		</HeaderBar>
        );
    };
export default Header;